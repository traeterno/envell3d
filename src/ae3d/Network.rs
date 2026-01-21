use std::time::{Duration, Instant};
use std::net::SocketAddr;
use std::io::{Read, Write};
use std::collections::HashMap;

use mio::{net::{TcpStream, UdpSocket}, Events, Interest, Poll, Registry, Token};

use crate::{ae3d::Window::Window, envell::message::{ToClient, ToServer}};

pub struct Network
{
	active: bool,
	ready: bool,
	tcp: Option<TcpStream>,
	udp: UdpSocket,
	tcpSequence: Vec<ToClient>,
	udpSequence: Vec<u8>,
	id: u8,
	tickRate: u8,
	state: HashMap<u8, (glam::Vec3, glam::Vec2)>,
	udpSock: SocketAddr,
	tcpSock: SocketAddr
}

impl Network
{
	pub fn init() -> Self
	{
		Self
		{
			active: false,
			ready: false,
			udp: UdpSocket::bind("0.0.0.0:0".parse().unwrap()).unwrap(),
			tcp: None,
			tcpSequence: vec![],
			udpSequence: vec![],
			id: u8::MAX,
			tickRate: 10,
			state: HashMap::new(),
			udpSock: "0.0.0.0:0".parse().unwrap(),
			tcpSock: "0.0.0.0:0".parse().unwrap()
		}
	}

	pub fn reset(&mut self)
	{
		self.active = false;
		if let Some(tcp) = self.tcp.as_mut()
		{
			let _ = tcp.shutdown(std::net::Shutdown::Both);
		}
	}

	pub fn connect(&mut self, ip: String) -> bool
	{
		let addr = ip.parse();
		if let Ok(addr) = addr
		{
			self.tcpSock = addr;
			let tcp = TcpStream::connect(addr);
			if let Ok(tcp) = tcp
			{
				self.tcp = Some(tcp);
				self.active = true;
				let _ = std::thread::Builder::new()
					.name(String::from("Network Update"))
					.spawn(update);
				return true;
			}
			println!("TCP: {}", tcp.unwrap_err());
			return false;
		}
		println!("\"{ip}\": {}", addr.unwrap_err());
		false
	}

	pub fn isReady(&self) -> bool { self.ready }
	pub fn isActive(&self) -> bool { self.active }
	pub fn getID(&self) -> u8 { self.id }

	pub fn send(&mut self, msg: ToServer)
	{
		if let Some(tcp) = self.tcp.as_mut()
		{
			let _ = tcp.write_all(&msg.toRaw());
		}
	}

	pub fn hasMessage(&self, topic: String) -> bool
	{
		for msg in &self.tcpSequence
		{
			match *msg
			{
				ToClient::Setup(..) => if topic == "setup" { return true; }
			}
		}
		false
	}

	pub fn getMessage(&mut self, topic: String) -> json::JsonValue
	{
		let mut out = json::JsonValue::Null;
		let mut index = usize::MAX;
		for i in 0..self.tcpSequence.len()
		{
			match self.tcpSequence[i]
			{
				ToClient::Setup(tickRate, id, port) if topic == "setup" =>
				{
					index = i;
					out = json::object!{
						tickRate: tickRate,
						id: id,
						port: port
					};
					break;
				}
				_ => {}
			}
		}
		if index != usize::MAX
		{
			self.tcpSequence.swap_remove(index);
		}
		out
	}

	pub fn setup(&mut self, tickRate: u8, id: u8, port: u16)
	{
		self.id = id;
		self.tickRate = tickRate;
		let ip = self.tcp.as_ref().unwrap().peer_addr().unwrap().ip().to_string();
		self.udpSock = format!("{ip}:{port}").parse().unwrap();
		self.ready = true;
		println!("Network is set up: {tickRate}|{port}|{id}");
	}

	pub fn getState(&self, id: u8) -> (glam::Vec3, glam::Vec2)
	{
		self.state.get(&id).cloned().unwrap_or_default()
	}

	pub fn setState(&mut self, pos: glam::Vec3, angle: glam::Vec2)
	{
		self.state.insert(self.id, (pos, angle));
	}

	pub fn discoveredIP(&mut self) -> String
	{
		if !self.udpSequence.is_empty()
		{
			return format!(
				"{}.{}.{}.{}:{}",
				self.udpSequence.remove(0),
				self.udpSequence.remove(0),
				self.udpSequence.remove(0),
				self.udpSequence.remove(0),
				u16::from_be_bytes([
					self.udpSequence.remove(0),
					self.udpSequence.remove(0),
				])
			);
		}
		String::new()
	}
}

fn update()
{
	let mut poll = Poll::new().unwrap();
	let mut events = Events::with_capacity(64);
	let n = Window::getNetwork();

	let mut tcpAttempt = 0;
	let tickTime = Duration::from_secs_f32(1.0 / n.tickRate as f32);
	let mut tickInstant = Instant::now();
	let evTime = Duration::from_secs_f32(0.1 / n.tickRate as f32);

	let _ = poll.registry().register(
		n.tcp.as_mut().unwrap(), Token(0),
		Interest::WRITABLE
	);
	
	while n.active
	{
		if tickInstant.elapsed() >= tickTime && n.ready
		{
			let s = n.state.get(&n.id).cloned().unwrap_or_default();
			match n.udp.send_to(&[
				&[n.id],
				&((s.0.x * 100.0) as i16).to_be_bytes() as &[u8],
				&((s.0.y * 100.0) as i16).to_be_bytes() as &[u8],
				&((s.0.z * 100.0) as i16).to_be_bytes() as &[u8],
				&(s.1.x as i16).to_be_bytes() as &[u8],
				&(s.1.y as i8).to_be_bytes() as &[u8],
			].concat(), n.udpSock)
			{
				Ok(_) => {}
				Err(x) =>
				{
					println!("Error when sending data on UDP:\n{x:#?}");
				}
			}
			tickInstant = Instant::now();
		}

		let _ = poll.poll(&mut events, Some(evTime));
		for e in events.iter()
		{
			if e.token().0 == 0
			{
				if e.is_writable()
				{
					match activate(n, poll.registry())
					{
						true => { tcpAttempt = u8::MAX; },
						false => tcpAttempt += 1
					};
					if tcpAttempt == 5 { n.active = false; }
					if tcpAttempt == u8::MAX
					{
						let _ = n.udp.set_broadcast(false);
					}
					continue;
				}
				
				if e.is_read_closed()
				{
					println!("Lost connection with server.");
					n.active = false;
					break;
				}
				
				let mut buf = vec![];
				let mut b = [0u8; 256];
				if let Ok(size) = n.tcp.as_mut().unwrap().read(&mut b)
				{
					buf.append(&mut b[..size].to_vec());
				}
				n.tcpSequence.append(&mut ToClient::fromRaw(buf));
			}
			if e.token().0 == 1
			{
				// xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx xxxxxxxx
				// 1  ]2        ]3       ]6     ]_____ 4                 5               ]
				// 1 - id, 2 - yaw, 3 - pitch, 4 - x, 5 - y, 6 - controls

				let mut buf = [0u8; 10];
				while let Ok(_) = n.udp.recv_from(&mut buf)
				{
					let id = buf[0];
					let x = i16::from_be_bytes([buf[1], buf[2]]);
					let y = i16::from_be_bytes([buf[3], buf[4]]);
					let z = i16::from_be_bytes([buf[5], buf[6]]);
					let yaw = i16::from_be_bytes([buf[7], buf[8]]);
					let pitch = i8::from_be_bytes([buf[9]]);
					n.state.insert(id, (
						glam::vec3(
							x as f32 * 0.01,
							y as f32 * 0.01,
							z as f32 * 0.01
						),
						glam::vec2(yaw as f32, pitch as f32)
					));
				}
			}
		}
	}
}

fn activate(n: &mut Network, reg: &Registry) -> bool
{
	let tcp = n.tcp.as_mut().unwrap();
	if tcp.peer_addr().is_err()
	{
		if let Ok(socket) = TcpStream::connect(n.tcpSock)
		{
			let _ = reg.deregister(tcp);
			*tcp = socket;
			let _ = reg.register(
				tcp, Token(0),
				Interest::WRITABLE
			);
		}
		return false;
	}
	let _ = reg.reregister(tcp, Token(0), Interest::READABLE);
	let _ = reg.register(&mut n.udp, Token(1), Interest::READABLE);
	n.send(ToServer::Setup(n.udp.local_addr().unwrap().port()));
	n.udpSequence.clear();
	true
}

pub fn search()
{
	let mut poll = Poll::new()
		.expect("Failed to create socket selector");
	let mut events = Events::with_capacity(16);

	let mut udp = UdpSocket::bind(
		"0.0.0.0:0".parse().unwrap()
	).expect("Failed to create UDP socket");
	udp.set_broadcast(true).expect("Failed to enable broadcast");

	poll.registry().register(
		&mut udp, Token(0),
		Interest::READABLE
	).expect("Failed to add UDP socket to registry");

	println!("Started searching...");

	let _ = udp.send_to(&[], "255.255.255.255:26225".parse().unwrap());
	
	'search: loop
	{
		let _ = poll.poll(
			&mut events,
			Some(Duration::from_secs(2))
		);
		
		if events.is_empty() { break 'search; }
		
		let mut buf = [0u8; 2];
		while let Ok((size, addr)) = udp.recv_from(&mut buf)
		{
			if size != 2 { continue; }
			match addr.ip()
			{
				std::net::IpAddr::V4(a) =>
				{
					Window::getNetwork().udpSequence.append(&mut [
						&a.octets() as &[u8],
						&buf as &[u8]
					].concat());
				}
				std::net::IpAddr::V6(_) =>
				{
					println!("Cannot use IPv6 yet.")
				}
			}
		}
	}

	println!("Stopped searching.");
}