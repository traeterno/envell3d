#[derive(Clone, Debug)]
pub struct Config
{
	pub tickRate: u8,
	pub firstCP: String,
	pub itemCellSize: u8,
	pub port: u16,
	pub playersCount: u8,
	pub sysTickRate: u16,
}

impl Default for Config
{
	fn default() -> Self
	{
		Self
		{
			tickRate: 10,
			firstCP: String::from(""),
			itemCellSize: 10,
			port: 26225,
			playersCount: 5,
			sysTickRate: 100
		}
	}
}

pub fn load(path: &str) -> Config
{
	let mut c = Config::default();
	if let Ok(f) = std::fs::read_to_string(path)
	{
		if let Ok(cfg) = json::parse(&f)
		{
			c.firstCP = cfg["firstCP"].as_str().unwrap_or("").to_string();
			c.itemCellSize = cfg["itemCellSize"].as_u8().unwrap_or(10);
			c.playersCount = cfg["playersCount"].as_u8().unwrap_or(5);
			c.port = cfg["port"].as_u16().unwrap_or(26225);
			c.tickRate = cfg["tickRate"].as_u8().unwrap_or(10);
			c.sysTickRate = cfg["sysTickRate"].as_u16().unwrap_or(100);
		}
	}
	c
}

pub fn save(cfg: &Config, path: &str)
{
	let _ = std::fs::write(path, json::stringify(json::object!{
		firstCP: cfg.firstCP.clone(),
		itemCellSize: cfg.itemCellSize,
		playersCount: cfg.playersCount,
		port: cfg.port,
		tickRate: cfg.tickRate,
	}));
}