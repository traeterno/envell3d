use mlua::{Function, Lua, Value};

use crate::ae3d::{Camera::Camera, Window::Window};

use super::{bind, Camera::Drawable, Sprite::Sprite, Text::Text};

pub struct Object
{
	name: String,
	script: Lua,
	spr: Sprite,
	text: Text
}

impl Object
{
	pub fn new() -> Self
	{
		Self
		{
			name: String::new(),
			script: Lua::new(),
			spr: Sprite::default(),
			text: Text::new()
		}
	}
	pub fn parse(node: &json::JsonValue) -> Self
	{
		let mut obj = Self
		{
			name: String::new(),
			script: Lua::new(),
			spr: Sprite::default(),
			text: Text::new()
		};

		bind::sprite(&obj.script);
		bind::text(&obj.script);
		bind::window(&obj.script);
		bind::world(&obj.script);
		bind::network(&obj.script);
		bind::profiler(&obj.script);
		bind::math(&obj.script);
		
		obj.name = node["name"].as_str().unwrap_or("???").to_string();
		obj.spr =
			if let Some(x) = node["image"].as_str()
			{
				Sprite::image(x.to_string())
			}
			else if let Some(x) = node["anim"].as_str()
			{
				Sprite::animated(x.to_string())
			}
			else { Sprite::default() };
		
		if let Some(x) = node["text"]["font"].as_str()
		{
			obj.text.setFont(x.to_string());
		}
		if let Some(x) = node["text"]["size"].as_f32()
		{
			obj.text.setSize(x);
		}
		if let Some(x) = node["text"]["text"].as_str()
		{
			obj.text.setString(x.to_string());
		}

		let t = obj.script.create_table().unwrap();
		for (var, value) in node["vars"].entries()
		{
			match value.as_f32()
			{
				Some(v) =>
				{
					let _ = t.raw_set(var, v);
				}
				None =>
				{
					let _ = t.raw_set(
						var, value.as_str().unwrap()
					);
				}
			};
		}
		let _ = obj.script.globals().set("vars", t);
		
		if let Ok(s) = std::fs::read_to_string(
			node["script"].as_str().unwrap_or("")
		)
		{
			let _ = obj.script.load_std_libs(mlua::StdLib::ALL_SAFE);
			let _ = obj.script.globals().raw_set(
				"ScriptID",
				format!("ui_{}", obj.name)
			);
			let _ = obj.script.load(s).exec();
		}
		
		obj
	}

	pub fn getSprite(&mut self) -> &mut Sprite
	{
		&mut self.spr
	}

	pub fn getText(&mut self) -> &mut Text
	{
		&mut self.text
	}

	pub fn getScript(&self) -> &mlua::Lua { &self.script }
}

pub struct UI
{
	baseSize: glam::Vec2,
	objects: Vec<Object>,
	reload: String,
	proj: glam::Mat4
}

impl UI
{
	pub fn init() -> Self
	{
		Self
		{
			baseSize: glam::Vec2::ZERO,
			objects: vec![],
			reload: String::new(),
			proj: glam::Mat4::IDENTITY
		}
	}

	pub fn setSize(&mut self, size: glam::Vec2)
	{
		self.baseSize = size;
	}

	pub fn load(&mut self, path: &str)
	{
		let src = json::parse(
			&std::fs::read_to_string(path)
			.unwrap_or(String::new())
		);
		if src.is_err()
		{
			println!("Failed to load UI: {}", src.unwrap_err());
			return;
		}
		let src = src.unwrap();

		self.objects.clear();

		for value in src.members()
		{
			self.objects.push(Object::parse(value));
		}

		for obj in &self.objects
		{
			let name = &obj.name;
			if let Ok(f) = obj.script.globals()
				.get::<mlua::Function>("Init")
			{
				match f.call::<mlua::Value>(())
				{
					Ok(_) => {},
					Err(x) =>
					{
						println!("Object Init: {name}\n{x}\n");
					}
				}
			}
		}

		self.resize();
	}

	pub fn getObject(&mut self, name: String) -> &mut Object
	{
		for o in &mut self.objects
		{
			if o.name == name { return o; }
		}
		panic!("UI object '{name}' not found");
	}

	pub fn update(&mut self)
	{
		Window::getProfiler().restart();
		for obj in &self.objects
		{
			let name = &obj.name;
			if let Ok(f) = obj.script.globals()
				.get::<mlua::Function>("Update")
			{
				match f.call::<mlua::Value>(())
				{
					Ok(_) => {},
					Err(x) =>
					{
						println!("Object Update: {name}\n{x}\n");
						let _ = obj.script.globals().raw_remove("Update");
					}
				}
			}
		}
		Window::getProfiler().save("uiUpdate".to_string());
	}

	pub fn requestLoad(&mut self, path: String)
	{
		self.reload = path;
	}

	pub fn updateReload(&mut self)
	{
		if !self.reload.is_empty()
		{
			self.load(&self.reload.clone());
			self.reload.clear();
		}
	}

	pub fn resize(&mut self)
	{
		let (w, h) = Window::getSize();
		let uiProj = glam::Mat4::orthographic_rh_gl(
			0.0, w as f32, h as f32, 0.0,
			-1.0, 1.0
		);
		let cam = Window::getCamera();
		cam.shaderUse("text");
		cam.shaderMat4("projection", uiProj);

		let sx = w as f32 / self.baseSize.x;
		let sy = h as f32 / self.baseSize.y;
		
		for obj in &self.objects
		{
			if let Ok(f) = obj.script.globals().get::<Function>("OnResized")
			{
				let _ = f.call::<Value>((sx, sy));
			}
		}
	}
	
	pub fn getSize(&self) -> glam::Vec2 { self.baseSize }
}

impl Drawable for UI
{
	fn draw(&mut self, _: &mut Camera)
	{
		Window::getProfiler().restart();
		for obj in &self.objects
		{
			let name = &obj.name;
			if let Ok(f) = obj.script.globals()
				.get::<mlua::Function>("Draw")
			{
				match f.call::<mlua::Value>(())
				{
					Ok(_) => {},
					Err(x) =>
					{
						println!("Object Draw: {name}\n{x}\n");
						let _ = obj.script.globals().raw_remove("Draw");
					}
				}
			}
		}
		unsafe { gl::Finish(); }
		Window::getProfiler().save("uiDraw".to_string());
	}
}