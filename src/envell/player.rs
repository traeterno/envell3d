
use std::{collections::HashMap, io::{Read, Write}, time::{Duration, Instant}};

use mio::{Events, Interest, Poll, Token, net::{TcpListener, TcpStream, UdpSocket}};

use crate::envell::{config::Config, message::{self, ToClient}};

struct Player
{
	tcp: TcpStream,
	ip: String,
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
	UnlockSettings(bool),
	ShowModal(usize, String),
	SetVisible(bool)
}

#[derive(Debug)]
pub enum Resp
{
	UpdateConfig(usize, Config),
	SetVisible(usize, bool)
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
			Resp::UpdateConfig(_, cfg) => { config = cfg; break }
			_ => {}
		}
	}
	println!("Session has acquired config.");

	let mut poll = Poll::new().expect("Failed to create socket selector");
	let mut events = Events::with_capacity(64);
	let mut players = Party::new();

	let mut listener;
	if let Ok(tcp) = TcpListener::bind(
		format!("0.0.0.0:{}", config.port).parse().unwrap()
	) { listener = tcp; }
	else if let Ok(tcp) = TcpListener::bind(
		"0.0.0.0:0".parse().unwrap()
	)
	{ listener = tcp; config.port = listener.local_addr().unwrap().port(); }
	else { panic!("Failed to create TCP listener at any port."); }

	let mut udp = UdpSocket::bind(
		"0.0.0.0:0".parse().unwrap()
	).expect("Failed to create UDP socket");

	let mut broadcast: Option<(UdpSocket, Instant)> = None;

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
		while let Ok((id, resp)) = fromMain.try_recv()
		{
			match resp
			{
				Resp::UpdateConfig(web, cfg) =>
				{
					if !players.is_empty()
					{
						let _ = toMain.send((
							id,
							Req::ShowModal(web, String::from("saveSettings-fail"))
						));
						continue;
					}
					println!("Player session: Config updated.");
					config = cfg;

					let _ = poll.registry().reregister(
						&mut listener, Token(config.playersCount as usize),
						Interest::READABLE
					);
					let _ = poll.registry().reregister(
						&mut udp, Token(config.playersCount as usize + 1),
						Interest::READABLE
					);

					let _ = toMain.send((
						id,
						Req::ShowModal(web, String::from("saveSettings-success"))
					));
				}
				Resp::SetVisible(web, active) =>
				{
					if !active
					{
						if let Some((s, _)) = broadcast.as_mut()
						{
							let _ = poll.registry().deregister(s);
							broadcast = None;
							println!("Broadcast shut down.");
							let _ = toMain.send((
								id, Req::ShowModal(web, String::from("setInvisible-success"))
							));
							let _ = toMain.send((0, Req::SetVisible(false)));
						}
						else
						{
							println!("Broadcast is already off.");
							let _ = toMain.send((
								id, Req::ShowModal(web, String::from("setInvisible-fail"))
							));
						}
						continue;
					}
					if broadcast.is_some()
					{
						println!("Broadcast is already active.");
						let _ = toMain.send((
							id, Req::ShowModal(web, String::from("setVisible-repeat"))
						));
						continue;
					}
					if let Ok(mut s) =
						UdpSocket::bind("0.0.0.0:26225".parse().unwrap())
					{
						if let Err(x) = s.set_broadcast(true)
						{
							println!("Cannot start broadcast: {x:?}");
							let _ = toMain.send((
								id,
								Req::ShowModal(web, String::from("setVisible-fail"))
							));
							continue;
						}
						let _ = poll.registry().register(
							&mut s, Token(config.playersCount as usize + 2),
							Interest::READABLE
						);
						broadcast = Some((s, Instant::now()));
						println!("Broadcast started.");
						let _ = toMain.send((
							id,
							Req::ShowModal(web, String::from("setVisible-success"))
						));
						let _ = toMain.send((0, Req::SetVisible(true)));
					}
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

		if let Some((s, t)) = broadcast.as_mut()
		{
			if t.elapsed().as_secs() > 60
			{
				println!("Broadcast time is out.");
				let _ = poll.registry().deregister(s);
				let _ = toMain.send((0, Req::SetVisible(false)));
				broadcast = None;
			}
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
					if players.len() == 0
					{
						let _ = toMain.send((socketID, Req::UnlockSettings(false)));
					}
					players.insert(id, Player
					{
						tcp: tcp,
						ip: ip.clone(),
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
			if socketID == config.playersCount + 2
			{
				if let Some((s, _)) = broadcast.as_mut()
				{
					let mut buf = [0u8; 2];
					while let Ok((size, addr)) = s.recv_from(&mut buf)
					{
						if size == 0
						{
							println!("Found searcher: {addr}");
							let _ = s.send_to(
								&config.port.to_be_bytes() as &[u8],
								addr
							);
						}
					}
					continue;
				}
			}

			let player = players.get_mut(&socketID).unwrap();

			if e.is_read_closed()
			{
				println!("Player #{socketID} has disconnected.");
				let _ = poll.registry().deregister(&mut player.tcp);
				players.remove(&socketID);
				// TODO broadcast disconnection to other players
				if players.len() == 0
				{
					let _ = toMain.send((socketID, Req::UnlockSettings(true)));
				}
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