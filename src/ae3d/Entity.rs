use mlua::Lua;

use crate::ae3d::{bind, Camera::{Camera, Drawable}, Mesh::Mesh};

pub struct Entity
{
	script: Lua,
	id: String,
	mesh: Mesh
}

impl Entity
{
	pub fn new() -> Self
	{
		Self
		{
			script: Lua::new(),
			id: String::new(),
			mesh: Mesh::new()
		}
	}

	pub fn load(id: String, path: String) -> Self
	{
		let mut ent = Self::new();
		
		bind::network(&ent.script);
		bind::world(&ent.script);
		bind::window(&ent.script);
		bind::shapes3D(&ent.script);
		bind::shaders(&ent.script);
		bind::mesh(&ent.script);

		let _ = ent.script.load(
			std::fs::read_to_string(
				path
			).unwrap()
		).exec();
		
		let _ = ent.script.globals().set(
			"ScriptID",
			format!("ent_{id}")
		);

		ent.id = id;

		ent
	}

	pub fn getScript(&self) -> &mlua::Lua
	{
		&self.script
	}

	pub fn init(&self, data: json::JsonValue)
	{
		let t = self.script.create_table().unwrap();
		
		for (var, value) in data.entries()
		{
			let _ = if value.is_number() { t.raw_set(var, value.as_f32().unwrap()) }
			else if value.is_boolean() { t.raw_set(var, value.as_bool().unwrap()) }
			else { t.raw_set(var, value.as_str().unwrap()) };
		}

		if let Ok(f) = self.script.globals().get::<mlua::Function>("Init")
		{
			let _ = f.call::<()>(t);
		}
	}

	pub fn update(&mut self)
	{
		bind::execFunc(&self.script, "Update");
	}

	pub fn getID(&self) -> String
	{
		self.id.clone()
	}

	pub fn getMesh(&mut self) -> &mut Mesh
	{
		&mut self.mesh
	}
}

impl Drawable for Entity
{
	fn draw(&mut self, _: &mut Camera)
	{
		bind::execFunc(&self.script, "Draw");
	}
}