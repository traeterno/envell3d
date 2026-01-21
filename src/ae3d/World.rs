use std::collections::HashMap;

use mlua::Lua;

use crate::ae3d::{bind, Camera::{Camera, Drawable}, Entity::Entity, Window::Window};

pub struct World
{
	name: String,
	script: Lua,
	ents: HashMap<String, Entity>,
	init: bool
}

impl World
{
	pub fn init() -> Self
	{
		Self
		{
			name: String::new(),
			script: Lua::new(),
			ents: HashMap::new(),
			init: true
		}
	}

	pub fn load(&mut self, id: String)
	{
		let path = String::from("res/scripts/worlds/") + &id + ".lua";
		self.parse(
			id,
			std::fs::read_to_string(path).unwrap_or_default()
		);
	}

	pub fn parse(&mut self, id: String, src: String)
	{
		self.name = id;
		self.script = Lua::new();
		self.ents.clear();

		match self.script.load(src).exec()
		{
			Ok(_) => {}
			Err(x) => { println!("Не удалось загрузить мир: {x}"); return; }
		}

		self.init = true;

		bind::window(&self.script);
		bind::network(&self.script);
		bind::world(&self.script);
		bind::shaders(&self.script);
		bind::camera(&self.script);
		bind::math(&self.script);
	}

	pub fn update(&mut self)
	{
		Window::getProfiler().restart();
		if self.init
		{
			bind::execFunc(&self.script, "Init");
			self.init = false;
		}
		bind::execFunc(&self.script, "Update");
		for (_, ent) in &mut self.ents
		{
			ent.update();
		}
		Window::getProfiler().save("worldUpdate".to_string());
	}

	pub fn getEntity(&mut self, id: String) -> &mut Entity
	{
		if let Some(e) = self.ents.get_mut(&id)
		{
			return e;
		}
		panic!("Entity '{id}' not found");
	}

	pub fn spawn(&mut self, id: String, path: String, vars: json::JsonValue)
	{
		self.ents.insert(id.clone(), Entity::load(id.clone(), path));
		self.ents.get_mut(&id).unwrap().init(vars);
	}

	pub fn kill(&mut self, id: String)
	{
		self.ents.remove(&id);
	}

	pub fn getName(&self) -> String { self.name.clone() }
}

impl Drawable for World
{
	fn draw(&mut self, cam: &mut Camera)
	{
		Window::getProfiler().restart();
		bind::execFunc(&self.script, "Draw");
		for (_, ent) in &mut self.ents
		{
			ent.draw(cam);
		}
		unsafe { gl::Finish(); }
		Window::getProfiler().save("worldDraw".to_string());
	}
}