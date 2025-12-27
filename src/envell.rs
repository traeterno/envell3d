// #![allow(non_snake_case, static_mut_refs)]
// // #![windows_subsystem = "windows"]
// mod server;
// use server::Server::Server;

// fn main()
// {
// 	let server = Server::getInstance();
// 	println!("Сервер запущен. Ждём игроков...");
// 	server.setStarted(false);
// 	server.setSilent(
// 		std::env::args().nth(1).unwrap_or_default() == "silent"
// 	);
// 	loop { server.update(); }
// }

use std::{collections::HashMap, io::{Read, Write}};

use mio::{net::{TcpListener, TcpStream}, Events, Interest, Poll, Token};

fn main()
{
	let mut poll = Poll::new().unwrap();
	let mut events = Events::with_capacity(64);
	let mut listener = TcpListener::bind(
		"0.0.0.0:2018".parse().unwrap()
	).unwrap();
	let _ = poll.registry().register(
		&mut listener,
		Token(0),
		Interest::READABLE
	);

	let mut connections: HashMap<Token, TcpStream> = HashMap::new();
	
	let mut counter = 1;

	loop
	{
		events.clear();
		let _ = poll.poll(&mut events, None);
		for e in events.iter()
		{
			if e.token().0 == 0
			{
				while let Ok((mut socket, addr)) = listener.accept()
				{
					println!("New connection from {addr}");
					let token = Token(counter);
					let _ = poll.registry().register(
						&mut socket, token,
						Interest::READABLE
					);
					connections.insert(token, socket);
					counter += 1;
				}
			}
			else
			{
				if let Some(s) = connections.get_mut(&e.token())
				{
					if e.is_read_closed()
					{
						let _ = poll.registry().deregister(s);
						connections.remove(&e.token());
						println!("Socket #{} has disconnected", e.token().0);
						if connections.len() == 0
						{
							println!("Every client has disconnected. Stopping...");
							std::process::exit(0);
						}
					}
					else if e.is_readable()
					{
						let mut buf = [0u8; 128];
						while let Ok(_) = s.read(&mut buf)
						{
							print!("The camera state: {};{};{}|{};{}   \r",
								f32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]).round(),
								f32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]).round(),
								f32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]).round(),
								f32::from_be_bytes([buf[12], buf[13], buf[14], buf[15]]).round(),
								f32::from_be_bytes([buf[16], buf[17], buf[18], buf[19]]).round()
							);
							let _ = std::io::stdout().flush();
						}
					}
				}
			}
		}
	}
}