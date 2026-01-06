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

	let state = state::load("res/system/save.json");

	let (
		mut toWeb,
		mut fromWeb
	) = launchWS();

	let (
		mut toSession,
		mut fromSession
	) = launchSession();

	let _ = toSession.send((0, player::Resp::UpdateConfig(cfg.clone())));

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
							let _ = toWeb.send((id, web::Resp::ChatLength(chat.len())));
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
							// TODO check active players
							config::apply(&mut cfg, new);
							sysTimer = Duration::from_secs_f32(
								1.0 / cfg.sysTickRate as f32
							);
							let _ = toSession.send(
								(0, player::Resp::UpdateConfig(cfg.clone()))
							);
							config::save(&cfg, "res/system/save.json");
							let _ = toWeb.send((id, web::Resp::SaveSettings(
								true,
								String::new()
							)));
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
				Ok((id, req)) =>
				{
					println!("{id}: {req:?}");
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
								(0, player::Resp::UpdateConfig(cfg.clone()))
							);
						}
					}
					break 'playerRecv;
				}
			}
		}

		if timer.elapsed() < sysTimer
		{
			std::thread::sleep(sysTimer - timer.elapsed());
		}
	}
}