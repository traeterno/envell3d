#![allow(non_snake_case)]

use std::time::{Duration, Instant};

mod config;
mod player;
mod state;
mod web;
pub mod message;

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

	let (toWeb, fromMainToWeb) =
		std::sync::mpsc::channel::<web::Response>();

	let (fromWebToMain, fromWeb) =
		std::sync::mpsc::channel::<web::Request>();

	let (toSession, fromMainToSession) =
		std::sync::mpsc::channel::<player::Response>();

	let (fromSessionToMain, fromSession) =
		std::sync::mpsc::channel::<player::Request>();

	let _ = std::thread::Builder::new()
		.name(String::from("WebServer"))
		.spawn(|| web::main(fromWebToMain, fromMainToWeb));

	let _ = std::thread::Builder::new()
		.name(String::from("Session"))
		.spawn(|| player::main(fromSessionToMain, fromMainToSession));

	let _ = toSession.send((0, player::Resp::UpdateConfig(cfg.clone())));

	let mut chat: Vec<(String, String)> = vec![];

	let mut sysTimer = Duration::from_secs_f32(1.0 / 100.0);

	loop
	{
		let timer = Instant::now();
		while let Ok((id, msg)) = fromWeb.try_recv()
		{
			match msg
			{
				web::Req::ChatMessages(offset) =>
				{
					let mut msgs =
						if offset >= chat.len() { vec![] }
						else { chat[offset..chat.len()].to_vec() };
					msgs.reverse();
					let _ = toWeb.send((id, web::Resp::ChatMessages(msgs)));
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
					let _ = toWeb.send((id, web::Resp::Settings(cfg.clone())));
				}
				web::Req::SaveSettings(new) =>
				{
					cfg.firstCP = new["firstCP"].as_str().unwrap_or("").to_string();
					cfg.itemCellSize = new["itemCellSize"].as_u8().unwrap_or(10);
					cfg.playersCount = new["playersCount"].as_u8().unwrap_or(5);
					cfg.port = new["port"].as_u16().unwrap_or(26225);
					cfg.tickRate = new["tickRate"].as_u8().unwrap_or(10);
					cfg.sysTickRate = new["sysTickRate"].as_u16().unwrap_or(100);
					sysTimer = Duration::from_secs_f32(1.0 / cfg.sysTickRate as f32);
					let _ = toSession.send((0, player::Resp::UpdateConfig(cfg.clone())));
					// let _ = toWeb.send((id, web::Resp::SaveSettings(
					// 	false,
					// 	String::from("ИДИ НАХУЙ УЁБИЩЕ")
					// )));
				}
			}
		}

		while let Ok((id, req)) = fromSession.try_recv()
		{
			println!("{id}: {req:?}");
		}

		if timer.elapsed() < sysTimer
		{
			std::thread::sleep(sysTimer - timer.elapsed());
		}
	}
}