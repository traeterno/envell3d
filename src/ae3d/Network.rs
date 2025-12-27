use std::{io::Write, net::TcpStream};

pub struct Network
{
	tcp: Option<TcpStream>,
}

impl Network
{
	pub fn init() -> Self
	{
		Self
		{
			tcp: None
		}
	}

	pub fn reset(&mut self)
	{
		if let Some(tcp) = &mut self.tcp
		{
			let _ = tcp.shutdown(std::net::Shutdown::Both);
		}
	}

	pub fn connect(&mut self, ip: String) -> bool
	{
		let addr = ip.parse();
		if let Ok(addr) = addr
		{
			let tcp = TcpStream::connect_timeout(
				&addr,
				std::time::Duration::from_secs(1)
			);
			if let Ok(tcp) = tcp
			{
				self.tcp = Some(tcp);
				println!("Connected to {ip}");
				return true;
			}
			println!("TCP: {}", tcp.unwrap_err());
			return false;
		}
		println!("\"{ip}\": {}", addr.unwrap_err());
		false
	}

	pub fn send(&mut self, buffer: Vec<u8>)
	{
		if let Some(tcp) = &mut self.tcp
		{
			let _ = tcp.write_all(&buffer);
		}
	}
}