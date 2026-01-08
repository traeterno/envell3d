#![allow(dead_code)]

pub enum ToServer
{
	Setup(u16)
}

impl ToServer
{
	pub fn fromRaw(buf: Vec<u8>) -> Vec<Self>
	{
		let mut out = vec![];

		let mut offset = 0;
		while buf.len() > offset
		{
			match buf[offset]
			{
				0 =>
				{
					out.push(Self::Setup(u16::from_be_bytes(
						[buf[offset + 1], buf[offset + 2]]
					)));
					offset += 3;
				}
				x => { println!("Unknown byte: {x}"); offset += 1; }
			}
		}
		
		out
	}

	pub fn toRaw(self) -> Vec<u8>
	{
		match self
		{
			Self::Setup(port) =>
			{
				[&[0], &port.to_be_bytes() as &[u8]].concat()
			}
		}
	}
}

pub enum ToClient
{
	Setup(u8, u8, u16)
}

impl ToClient
{
	pub fn fromRaw(buf: Vec<u8>) -> Vec<Self>
	{
		let mut out = vec![];
		let mut offset = 0;
		while buf.len() > offset
		{
			match buf[offset]
			{
				0 =>
				{
					out.push(Self::Setup(
						buf[offset + 1],
						buf[offset + 2],
						u16::from_be_bytes([buf[offset + 3], buf[offset + 4]]),
					));
					offset += 5;
				}
				x => { println!("Unknown byte: {x}"); offset += 1; }
			}
		}
		out
	}

	pub fn toRaw(self) -> Vec<u8>
	{
		match self
		{
			Self::Setup(tickRate, id, port) =>
			{
				[&[0, tickRate, id], &port.to_be_bytes() as &[u8]].concat()
			}
		}
	}
}