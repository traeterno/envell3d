use std::{collections::HashMap, io::{Read, Write}};

use base64::{prelude::BASE64_STANDARD, Engine};
use mio::{net::{TcpListener, TcpStream}, Events, Interest, Poll, Token};

use crate::envell::{config::Config, state::State};

#[derive(PartialEq, Debug)]
enum ClientMode
{
	Http,
	WebSocket,
	Disconnected
}

pub enum Req
{
	ChatMessages(usize),
	NewMessage(String),
	State,
	GetSettings,
	SaveSettings(json::JsonValue)
}

pub type Request = (usize, Req);

pub enum Resp
{
	ChatMessages(Vec<(String, String)>),
	NewMessage(String, String),
	State(State),
	GetSettings(Config),
	ChatLength(usize),
	SaveSettings(bool, String)
}

pub type Response = (usize, Resp);

pub fn main(
	toMain: std::sync::mpsc::Sender<Request>,
	fromMain: std::sync::mpsc::Receiver<Response>
)
{
	let mut listener = TcpListener::bind(
		"0.0.0.0:8080".parse().unwrap()
	);
	if listener.is_err() { listener = TcpListener::bind("0.0.0.0:0".parse().unwrap()); }

	let mut listener = listener.expect("Failed to create web server.");

	println!("Launched web server on port {}.", listener.local_addr().unwrap().port());

	let mut poll = Poll::new().expect("Failed to create socket selector.");
	let _ = poll.registry().register(
		&mut listener, Token(0),
		Interest::READABLE
	);

	let mut events = Events::with_capacity(64);

	let mut token = 1;
	let mut clients =
		HashMap::<Token, (ClientMode, TcpStream)>::new();

	loop
	{
		while let Ok(msg) = fromMain.try_recv()
		{
			match msg.1
			{
				Resp::NewMessage(user, msg) =>
				{
					for (_, (m, c)) in &mut clients
					{
						if *m == ClientMode::WebSocket
						{
							sendWS(c, Resp::NewMessage(user.clone(), msg.clone()));
						}
					}
				}
				Resp::ChatLength(l) =>
				{
					for (t, (m, c)) in &mut clients
					{
						if *m == ClientMode::WebSocket && msg.0 != t.0
						{
							sendWS(c, Resp::ChatLength(l));
						}
					}
				}
				resp =>
				{
					if let Some((_, tcp)) = clients.get_mut(&Token(msg.0))
					{
						sendWS(tcp, resp);
					}
				}
			}
		}

		let _ = poll.poll(
			&mut events,
			Some(std::time::Duration::from_millis(20))
		);

		for e in events.iter()
		{
			let socketID = e.token().0;
			if socketID == 0
			{
				while let Ok((mut tcp, _)) = listener.accept()
				{
					let t = Token(token);
					let _ = poll.registry().register(
						&mut tcp, t,
						Interest::READABLE
					);
					clients.insert(t, (ClientMode::Http, tcp));
					token += 1;
				}
			}
			else
			{
				let client = clients.get_mut(&e.token()).unwrap();

				if e.is_read_closed()
				{
					let _ = poll.registry().deregister(&mut client.1);
					clients.remove(&Token(socketID));
					continue;
				}
				
				let mut buf = [0u8; 1024];
				while let Ok(size) = client.1.read(&mut buf)
				{
					match client.0
					{
						ClientMode::Http =>
						{
							match handleHTTP(&mut client.1, &buf[..size])
							{
								0 => {}
								1 => { client.0 = ClientMode::Disconnected; break; }
								2 =>
								{
									if setupWS(
										&mut client.1, &buf[..size],
										socketID, &toMain
									) { client.0 = ClientMode::WebSocket; }
									else { client.0 = ClientMode::Disconnected; break; }
								}
								x => println!("Unknown HTTP status code: {x}")
							}
						}
						ClientMode::WebSocket =>
						{
							if size == 0 { client.0 = ClientMode::Disconnected; break; }
							if let Some((msg, data)) = receiveWS(&buf[..size])
							{
								handleWS(socketID, &msg, data, &toMain);
							}
							else { client.0 = ClientMode::Disconnected; break; }
						}
						ClientMode::Disconnected => {}
					}
				}
				if client.0 == ClientMode::Disconnected
				{
					let _ = poll.registry().deregister(&mut client.1);
					clients.remove(&Token(socketID));
					if clients.len() == 0 { token = 1; }
				}
			}
		}
	}
}

fn handleHTTP(tcp: &mut TcpStream, buf: &[u8]) -> u8
{
	let raw = String::from_utf8_lossy(buf).to_string();
	let args: Vec<&str> = raw.split("\r\n").collect();
	let mut path = args[0].split(" ").collect::<Vec<&str>>()[1];

	if path == "/ws" { return 2; }
	if path == "/" { path = "/index.html"; }

	let (mimetype, bin) = match path.split(".").last().unwrap()
	{
		"html" => ("text/html", false),
		"css" => ("text/css", false),
		"js" => ("text/javascript", false),
		"png" => ("image/png", true),
		"otf" => ("application/x-font-opentype", true),
		x => { println!("{x}"); ("", false) }
	};

	let (found, data) =
		match std::fs::read(String::from("res/web/") + path)
	{
		Ok(f) => { (true, f) }
		Err(x) => { println!("{x:#?}"); (false, vec![]) }
	};

	let _ = tcp.write_all(&match data.is_empty()
	{
		true => "HTTP/1.1 404 Not Found".as_bytes().to_vec(),
		false => [
			(String::from("HTTP/1.1 200 OK") +
			"\r\nConnection: keep-alive" +
			"\r\nContent-Type: " + mimetype +
			if bin { "" } else { "; charset=UTF-8" } +
			"\r\nContent-Length: " + &data.len().to_string() +
			"\r\n\r\n").as_bytes().to_vec(),
			data,
			"\r\n\r\n".as_bytes().to_vec()
		].concat()
	});

	if found { 0 } else { 1 }
}

fn setupWS(
	tcp: &mut TcpStream,
	buf: &[u8],
	id: usize,
	toMain: &std::sync::mpsc::Sender<Request>
) -> bool
{
	let raw = String::from_utf8_lossy(buf).to_string();
	let args: Vec<(&str, &str)> = raw.split("\r\n").map(|x|
	{
		if !x.contains(": ") { return ("", ""); }
		let t: Vec<&str> = x.split(": ").collect();
		(t[0], t[1])
	}).filter(|(a, b)| !a.is_empty() && !b.is_empty()).collect();
	let args = {
		let mut h = HashMap::new();
		for (var, value) in args
		{
			h.insert(var.to_lowercase(), value);
		}
		h
	};

	let key = args.get("sec-websocket-key").unwrap_or(&"").to_string();
	if key.is_empty() { println!("No key is provided."); return false; }

	let magic = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
	let enc = BASE64_STANDARD.encode(
		sha1_smol::Sha1::from(key + magic).digest().bytes()
	);
	let _ = tcp.write_all((
		String::from("HTTP/1.1 101 Switching Protocols") +
		"\r\nUpgrade: websocket\r\nConnection: Upgrade" +
		"\r\nSec-WebSocket-Accept: " + &enc + "\r\n\r\n"
	).as_bytes());

	let _ = toMain.send((id, Req::State));
	let _ = toMain.send((id, Req::GetSettings));
	
	true
}

fn sendWS(tcp: &mut TcpStream, msg: Resp)
{
	let topic: &'static str;
	let mut obj: json::JsonValue;
	match msg
	{
		Resp::ChatMessages(history) =>
		{
			topic = "chatMessages";
			obj = json::array![];
			for (user, msg) in history
			{
				let _ = obj.push(json::object!{
					user: user,
					msg: msg
				});
			}
		}
		Resp::NewMessage(user, msg) =>
		{
			topic = "chatMessages";
			obj = json::array![
				{
					user: user.clone(),
					msg: msg.clone()
				}
			];
		}
		Resp::State(state) =>
		{
			topic = "state";
			obj = json::array![
				{
					title: "Сохранение",
					props: {
						"Чекпоинт": state.checkpoint.clone(),
						"Дата сохранения": state.date.clone()
					}
				}
			];
		}
		Resp::GetSettings(cfg) =>
		{
			topic = "getSettings";
			obj = json::object!{
				"Сервер": {
					tickRate: {
						type: "range",
						name: "Частота синхронизации игроков",
						value: cfg.tickRate,
						props: { min: 1, max: 100 }
					},
					firstCP: {
						type: "string",
						name: "Первый чекпоинт",
						value: cfg.firstCP.clone()
					},
					itemCellSize: {
						type: "range",
						name: "Количество предметов в ячейке",
						value: cfg.itemCellSize,
						props: { min: 1, max: 255 }
					},
					playersCount: {
						type: "range",
						name: "Количество игроков",
						value: cfg.playersCount,
						props: { min: 1, max: 32 }
					},
					port: {
						type: "range",
						name: "Порт сервера",
						value: cfg.port,
						props: { min: 1024, max: 65535 }
					},
					sysTickRate: {
						type: "range",
						name: "Частота обновления сервера",
						value: cfg.sysTickRate,
						props: { min: 1, max: 1024 }
					}
				}
			};
		}
		Resp::ChatLength(l) =>
		{
			topic = "chatLength";
			obj = json::from(l);
		}
		Resp::SaveSettings(result, reason) =>
		{
			topic = "saveSettings";
			obj = json::object!{
				ok: result,
				reason: reason.clone()
			};
		}
	}

	let raw = json::stringify(json::object!{ type: topic, data: obj });

	let len =
		if raw.len() <= 125 { vec![raw.len() as u8] }
		else { [&[126u8] as &[u8], &(raw.len() as u16).to_be_bytes()].concat() };

	let _ = tcp.write_all(&[
		&[0b10_00_00_01 as u8],
		len.as_slice(),
		raw.as_bytes()
	].concat());
}

fn receiveWS(buf: &[u8]) -> Option<(String, json::JsonValue)>
{
	let isFinal = buf[0] & 0b10_00_00_00 == 128;
	let kind = buf[0] & 0b00_00_11_11;
	if kind == 8 { return None; }
	let (payloadLength, offset) = {
		let check = buf[1] & 0b01_11_11_11;
		if check <= 125 { (check as u64, 2) }
		else if check == 126
		{
			(u16::from_be_bytes([buf[2], buf[3]]) as u64, 4)
		}
		else
		{
			(u64::from_be_bytes([
				buf[2], buf[3], buf[4], buf[5],
				buf[6], buf[7], buf[8], buf[9]
			]), 10)
		}
	};
	let key = &buf[offset..offset + 4];
	if !isFinal { println!("Cannot process partial packet."); return None; }
	let mut raw = vec![];
	for i in 0..payloadLength as usize
	{
		raw.push(buf[offset + 4 + i] ^ key[i % 4]);
	}
	let msg = json::parse(
		&String::from_utf8_lossy(&raw).to_string()
	).unwrap_or(json::object!{ "invalid": {} });

	let (msg, data) = msg.entries().nth(0).unwrap();
	Some((msg.to_string(), data.to_owned()))
}

fn handleWS(
	id: usize,
	msg: &str,
	data: json::JsonValue,
	toMain: &std::sync::mpsc::Sender<Request>
)
{
	match msg
	{
		"chatMessages" =>
		{
			let _ = toMain.send((id,
				Req::ChatMessages(data["messagesLength"].as_usize().unwrap_or(0))
			));
		}
		"newMessage" =>
		{
			let _ = toMain.send((id,
				Req::NewMessage(data["msg"].as_str().unwrap_or("").to_string())
			));
		}
		"saveSettings" =>
		{
			let _ = toMain.send((id, Req::SaveSettings(data)));
		}
		x =>
		{
			println!("Unknown request: {x}");
			println!("{data:#}");
		}
	}
}