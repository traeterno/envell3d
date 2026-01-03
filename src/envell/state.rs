use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Account
{
	name: String,
	class: String,
	color: (u8, u8, u8),
	inventory: Vec<(String, u8)>
}

impl Default for Account
{
	fn default() -> Self
	{
		Self
		{
			name: String::new(),
			class: String::new(),
			color: (255, 255, 255),
			inventory: vec![(String::new(), 0); 8]
		}
	}
}

type Players = HashMap<String, Account>;

#[derive(Clone, Default, Debug)]
pub struct State
{
	pub date: String,
	pub checkpoint: String,
	pub players: Players
}

impl State
{
	pub fn getAccount(&self, ip: String) -> Account
	{
		self.players.get(&ip).cloned().unwrap_or_default()
	}
}

pub fn load(path: &str) -> State
{
	let mut s = State::default();

	if let Ok(f) = std::fs::read_to_string(path)
	{
		if let Ok(state) = json::parse(&f)
		{
			s.date = state["date"].as_str().unwrap_or("???").to_string();
			s.checkpoint = state["checkpoint"].as_str().unwrap_or("???").to_string();
			s.players = parsePlayers(&state["players"]);
		}
	}
	
	s
}

fn parsePlayers(src: &json::JsonValue) -> Players
{
	let mut out = Players::new();
	if src.is_null() { return out; }
	for p in src.members()
	{
		let color = &p["color"];
		let mut inventory = vec![];
		for i in 0..8
		{
			inventory.push(
				(
					p["inventory"][i]["id"].as_str().unwrap_or("").to_string(),
					p["inventory"][i]["count"].as_u8().unwrap_or(0)
				)
			);
		}
		out.insert(
			p["ip"].as_str().unwrap_or("0.0.0.0").to_string(),
			Account
			{
				name: p["name"].as_str().unwrap_or("NoName").to_string(),
				class: p["class"].as_str().unwrap_or("Unknown").to_string(),
				color: (
					color[0].as_u8().unwrap_or(255),
					color[1].as_u8().unwrap_or(255),
					color[2].as_u8().unwrap_or(255)
				),
				inventory: inventory
			}
		);
	}
	out
}