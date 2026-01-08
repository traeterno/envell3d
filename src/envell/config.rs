#[derive(Clone, Debug)]
pub struct Config
{
	pub tickRate: u8,
	pub firstCP: String,
	pub itemCellSize: u8,
	pub port: u16,
	pub playersCount: u8,
	pub sysTickRate: u16,
	pub locked: bool
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
			sysTickRate: 100,
			locked: false
		}
	}
}

pub fn apply(cfg: &mut Config, data: json::JsonValue)
{
	cfg.firstCP = data["firstCP"].as_str().unwrap_or("").to_string();
	cfg.itemCellSize = data["itemCellSize"].as_u8().unwrap_or(10);
	cfg.playersCount = data["playersCount"].as_u8().unwrap_or(5);
	cfg.port = data["port"].as_u16().unwrap_or(26225);
	cfg.tickRate = data["tickRate"].as_u8().unwrap_or(10);
	cfg.sysTickRate = data["sysTickRate"].as_u16().unwrap_or(100);
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

pub fn settings(cfg: &Config) -> json::JsonValue
{
	json::object!{
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
	}
}