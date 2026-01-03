
use std::{collections::HashMap, io::{Read, Write}, time::{Duration, Instant}};

use mio::{Events, Interest, Poll, Token, net::{TcpListener, TcpStream, UdpSocket}};

use crate::envell::{config::Config, message::{self, ToClient}, state::Account};

struct Player
{
	tcp: TcpStream,
	ip: String,
	info: Account,
	udpPort: u16,
	state: [u8; 9] // TODO rewrite
}

impl Player
{
	pub fn send(&mut self, msg: ToClient)
	{
		let _ = self.tcp.write_all(&msg.toRaw());
	}
}

type Party = HashMap<u8, Player>;

#[derive(Debug)]
pub enum Req
{
	GetPlayerInfo(String, u8),
	Login(String)
}

#[derive(Debug)]
pub enum Resp
{
	PlayerInfo(u8, Account),
	UpdateConfig(Config)
}

pub type Request = (u8, Req);
pub type Response = (u8, Resp);

pub fn main(
	toMain: std::sync::mpsc::Sender<Request>,
	fromMain: std::sync::mpsc::Receiver<Response>
)
{
	let mut config = Config::default();
	while let Ok((_, resp)) = fromMain.recv()
	{
		match resp
		{
			Resp::UpdateConfig(cfg) => { config = cfg; break }
			_ => {}
		}
	}
	println!("Session has acquired config.");

	let mut poll = Poll::new().expect("Failed to create socket selector");
	let mut events = Events::with_capacity(64);
	let mut players = Party::new();

	let mut listener = TcpListener::bind(
		format!("0.0.0.0:{}", config.port).parse().unwrap()
	).expect(&format!("Failed to bind listener to port {}", config.port));

	let mut udp = UdpSocket::bind(
		"0.0.0.0:0".parse().unwrap()
	).expect("Failed to create UDP socket");

	println!("TCP: {} | UDP: {}",
		listener.local_addr().unwrap(),
		udp.local_addr().unwrap()
	);

	let _ = poll.registry().register(
		&mut listener, Token(config.playersCount as usize),
		Interest::READABLE
	);

	let _ = poll.registry().register(
		&mut udp, Token(config.playersCount as usize + 1),
		Interest::READABLE
	);

	let tickTime = Duration::from_secs_f32(1.0 / config.tickRate as f32);
	let mut tickTimer = Instant::now();

	loop
	{
		while let Ok((_, resp)) = fromMain.try_recv()
		{
			match resp
			{
				Resp::UpdateConfig(cfg) =>
				{
					if !players.is_empty()
					{
						// TODO block ability to update settings when someone is in game
						println!("Can't apply config, someone is in game.");
						continue;
					}
					println!("Player session: Config updated.");
					config = cfg;

					let _ = poll.registry().reregister(
						&mut listener, Token(config.playersCount as usize),
						Interest::READABLE
					);
				}
				Resp::PlayerInfo(id, acc) =>
				{
					// 
				}
			}
		}

		if tickTimer.elapsed() >= tickTime
		{
			for (id1, p1) in &players
			{
				for (id2, p2) in &players
				{
					if *id1 == *id2 { continue; }
					let _ = udp.send_to(
						&[&[*id1] as &[u8], &p1.state].concat(),
						format!("{}:{}", p2.ip, p2.udpPort).parse().unwrap()
					);
				}
			}
			tickTimer = Instant::now();
		}

		let _ = poll.poll(&mut events,
			Some(std::time::Duration::from_millis(20))
		);

		for e in events.iter()
		{
			let token = e.token();
			let socketID = token.0 as u8;
			if socketID == config.playersCount
			{
				while let Ok((mut tcp, addr)) = listener.accept()
				{
					let id = getEmptyID(&players, config.playersCount);
					println!("New player #{id}: {addr}");
					let _ = poll.registry().register(
						&mut tcp, Token(id as usize),
						Interest::READABLE
					);
					let ip = tcp.peer_addr().unwrap().ip().to_string();
					players.insert(id, Player
					{
						tcp: tcp,
						ip: ip.clone(),
						info: Account::default(),
						udpPort: 0,
						state: [0u8; 9]
					});
				}
				continue;
			}
			if socketID == config.playersCount + 1
			{
				let mut buf = [0u8; 128];
				'udp: loop
				{
					match udp.recv_from(&mut buf)
					{
						Ok(_) =>
						{
							if let Some(p) = players.get_mut(&buf[0])
							{
								p.state = [
									buf[1], buf[2],
									buf[3], buf[4],
									buf[5], buf[6],
									buf[7], buf[8],
									buf[9]
								];
							}
							else
							{
								println!("P{} not found", buf[0]);
							}
						}
						Err(x) =>
						{
							if x.kind() == std::io::ErrorKind::WouldBlock { break 'udp; }
							println!("Server UDP: {x}");
							let _ = poll.registry().reregister(
								&mut udp, Token(config.playersCount as usize + 1),
								Interest::READABLE
							);
							break 'udp;
						}
					}
				}
				continue;
			}

			let player = players.get_mut(&socketID).unwrap();

			if e.is_read_closed()
			{
				println!("Player #{socketID} has disconnected.");
				let _ = poll.registry().deregister(&mut player.tcp);
				players.remove(&socketID);
				// TODO broadcast disconnection to other players
				continue;
			}

			let mut buf = [0u8; 1024];
			let mut out = vec![];
			while let Ok(size) = player.tcp.read(&mut buf)
			{
				out = [out, buf[..size].to_vec()].concat();
			}
			for msg in message::ToServer::fromRaw(out)
			{
				match msg
				{
					message::ToServer::Setup(port) =>
					{
						player.udpPort = port;
						player.send(ToClient::Setup(
							config.tickRate,
							socketID,
							udp.local_addr().unwrap().port()
						));
					}
				}
			}
		}
	}
}

fn getEmptyID(party: &Party, count: u8) -> u8
{
	for i in 0..count
	{
		if !party.contains_key(&i)
		{
			return i;
		}
	}
	u8::MAX
}