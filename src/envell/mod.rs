#![allow(non_snake_case)]

use std::time::{Duration, Instant};

mod config;
mod player;
mod state;
mod web;
pub mod message;

fn launchWS() -> (
	std::sync::mpsc::Sender<web::Response>,
	std::sync::mpsc::Receiver<web::Request>
)
{
	let (toWeb, fromMainToWeb) =
		std::sync::mpsc::channel::<web::Response>();

	let (fromWebToMain, fromWeb) =
		std::sync::mpsc::channel::<web::Request>();
	
	let _ = std::thread::Builder::new()
		.name(String::from("WebServer"))
		.spawn(|| web::main(fromWebToMain, fromMainToWeb));
	(toWeb, fromWeb)
}

fn launchSession() -> (
	std::sync::mpsc::Sender<player::Response>,
	std::sync::mpsc::Receiver<player::Request>
)
{
	let (toSession, fromMainToSession) =
		std::sync::mpsc::channel::<player::Response>();

	let (fromSessionToMain, fromSession) =
		std::sync::mpsc::channel::<player::Request>();

	let _ = std::thread::Builder::new()
		.name(String::from("Session"))
		.spawn(|| player::main(fromSessionToMain, fromMainToSession));

	(toSession, fromSession)
}

pub fn main()
{
	let mut cfg = config::load("res/system/cfg.json");
	if cfg.firstCP.is_empty()
	{
		println!("No configuration found. Creating new one.");
		println!("Proceed to http://localhost:8080 and set the server up.");
		config::save(&cfg, "res/system/cfg.json");
	}
	else { println!("Configuration found."); }

	let mut state = state::load("res/system/save.json");

	let (
		mut toWeb,
		mut fromWeb
	) = launchWS();

	let (
		mut toSession,
		mut fromSession
	) = launchSession();

	let _ = toSession.send((0, player::Resp::UpdateConfig(usize::MAX, cfg.clone())));

	let mut chat: Vec<(String, String)> = vec![];

	let mut sysTimer = Duration::from_secs_f32(1.0 / 100.0);

	loop
	{
		let timer = Instant::now();
		'webRecv: loop
		{
			match fromWeb.try_recv()
			{
				Ok((id, msg)) =>
				{
					match msg
					{
						web::Req::ChatMessages(offset) =>
						{
							let mut msg =
								if offset >= chat.len() { vec![] }
								else { chat[offset..chat.len()].to_vec() };
							msg.reverse();
							let _ = toWeb.send((id, web::Resp::ChatMessages(msg)));
						}
						web::Req::NewMessage(msg) =>
						{
							println!("WebClient #{id}: {msg}");
							let name = format!("WebClient #{id}");
							chat.push((name.clone(), msg.clone()));
							let _ = toWeb.send((id, web::Resp::NewMessage(name, msg)));
						}
						web::Req::State =>
						{
							let _ = toWeb.send((id, web::Resp::State(state.clone())));
						}
						web::Req::GetSettings =>
						{
							let _ = toWeb.send((id, web::Resp::GetSettings(cfg.clone())));
						}
						web::Req::SaveSettings(new) =>
						{
							if cfg.locked
							{
								let _ = toWeb.send(
									(id, web::Resp::Modal("saveSettings-fail".to_string()))
								);
								continue;
							}
							config::apply(&mut cfg, new);
							config::save(&cfg, "res/system/save.json");
							sysTimer = Duration::from_secs_f32(
								1.0 / cfg.sysTickRate as f32
							);
							let _ = toSession.send(
								(0, player::Resp::UpdateConfig(id, cfg.clone()))
							);
							let _ = toWeb.send(
								(id, web::Resp::Modal("saveSettings-success".to_string()))
							);
						}
						web::Req::Modal(id, result) =>
						{
							match id.as_str()
							{
								"saveSettings-success" => { println!("Settings saved."); }
								"saveSettings-fail" => { println!("Failed to save settings."); }
								"stopServer" =>
								{
									if let Some(x) = result["pwd"].as_str()
									{
										if x == cfg.password
										{
											println!("Correct password. Stopping the server...");
											std::process::exit(0);
										}
										else
										{
											println!("Incorrect password to stop the server.");
										}
									}
									else
									{
										println!("Revoked request to stop the server.");
									}
								}
								x => { println!("New modal: {x}: {result:#}") }
							}
						}
						web::Req::Buttons =>
						{
							let _ = toWeb.send((id, web::Resp::Buttons(vec![
								(String::from("setVisible"), String::from("Открыть врата")),
								(String::from("setInvisible"), String::from("Закрыть врата")),
								(String::from("stop"), String::from("Остановить сервер"))
							])));
						}
						web::Req::ClickButton(btn) =>
						{
							if btn == "setVisible"
							{
								let _ = toSession.send((0, player::Resp::SetVisible(id, true)));
							}
							if btn == "setInvisible"
							{
								let _ = toSession.send((0, player::Resp::SetVisible(id, false)));
							}
							if btn == "stop"
							{
								let _ = toWeb.send((
									id, web::Resp::Modal("stopServer".to_string())
								));
							}
						}
					}
				}
				Err(x) =>
				{
					match x
					{
						std::sync::mpsc::TryRecvError::Empty => {}
						std::sync::mpsc::TryRecvError::Disconnected =>
						{
							println!("WebServer channel has disconnected. Reloading...");
							(toWeb, fromWeb) = launchWS();
						}
					}
					break 'webRecv;
				}
			}
		}

		'playerRecv: loop
		{
			match fromSession.try_recv()
			{
				Ok((_, req)) =>
				{
					match req
					{
						player::Req::UnlockSettings(active) =>
						{
							cfg.locked = active;
						}
						player::Req::ShowModal(web, id) =>
						{
							let _ = toWeb.send((
								web,
								web::Resp::Modal(id)
							));
						}
						player::Req::SetVisible(active) =>
						{
							state.visible = active;
							let _ = toWeb.send((0, web::Resp::State(state.clone())));
						}
					}
				}
				Err(x) =>
				{
					match x
					{
						std::sync::mpsc::TryRecvError::Empty => {}
						std::sync::mpsc::TryRecvError::Disconnected =>
						{
							println!("Player session channel has disconnected. Reloading...");
							(toSession, fromSession) = launchSession();
							let _ = toSession.send(
								(0, player::Resp::UpdateConfig(usize::MAX, cfg.clone()))
							);
						}
					}
					break 'playerRecv;
				}
			}
		}

		if timer.elapsed() < sysTimer
		{
			if let Some(x) = sysTimer.checked_sub(timer.elapsed())
			{
				std::thread::sleep(x);
			}
		}
	}
}