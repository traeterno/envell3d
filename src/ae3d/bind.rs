use mlua::{Lua, Table};

use crate::ae3d::{glTF::GLTF, Mesh::Mesh, Skeleton::Skeleton};
use crate::ae3d::{Entity::Entity, Programmable::Variable, World::World};

use super::{Sprite::Sprite, Text::Text, Window::Window};

fn getScript(id: String) -> &'static mlua::Lua
{
	let mut id = id.split("_");
	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getScript(),
		"ent" => Window::getWorld().getEntity(id.nth(0).unwrap().to_string()).getScript(),
		x => panic!("Script Lua: {x} not defined")
	}
}

fn getSprite(id: String) -> &'static mut Sprite
{
	let mut id = id.split("_");

	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getSprite(),
		x => panic!("Sprite Lua: {x} not defined")
	}
}

fn getMesh(id: String) -> &'static mut Mesh
{
	let mut id = id.split("_");

	match id.nth(0).unwrap()
	{
		"ent" => Window::getWorld().getEntity(id.nth(0).unwrap().to_string()).getMesh(),
		x => panic!("Sprite Lua: {x} not defined")
	}
}

fn getSkeleton(id: String) -> &'static mut Skeleton
{
	let mut id = id.split("_");

	match id.nth(0).unwrap()
	{
		"ent" => Window::getWorld().getEntity(id.nth(0).unwrap().to_string()).getSkeleton(),
		x => panic!("Sprite Lua: {x} not defined")
	}
}

fn getText(id: String) -> &'static mut Text
{
	let mut id = id.split("_");
	
	match id.nth(0).unwrap()
	{
		"ui" => Window::getUI().getObject(id.nth(0).unwrap().to_string()).getText(),
		x => panic!("Text Lua: {x} not defined")
	}
}

fn getEntity(s: &Lua) -> &'static mut Entity
{
	let id: String = s.globals().get("ScriptID").unwrap();
	Window::getWorld().getEntity(id.split("_").nth(1).unwrap().to_string())
}

pub fn execFunc(script: &Lua, func: &str)
{
	if let Ok(f) = script.globals().raw_get::<mlua::Function>(func)
	{
		match f.call::<mlua::Value>(())
		{
			Ok(_) => {}
			Err(x) =>
			{
				println!("Failed to call '{func}' function:\n{x}");
				println!("Script: {}\n",
					script.globals().raw_get::<String>("ScriptID").unwrap()
				);
				let _ = script.globals().raw_remove(func);
			}
		}
	}
}

fn func<F, A, R>(s: &Lua, t: &mlua::Table, name: &str, f: F)
where
	F: Fn(&Lua, A) -> mlua::Result<R>
		+ mlua::MaybeSend + 'static,
	A: mlua::FromLuaMulti, R: mlua::IntoLuaMulti
{
	let _ = t.raw_set(name, s.create_function(f).unwrap());
}

pub fn sprite(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		Window::getCamera().draw(spr);
		Ok(())
	}).unwrap());

	let _ = t.set("size",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = spr.getFrameSize();
		Ok((s.x, s.y))
	}).unwrap());

	let _ = t.set("texSize",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = spr.getTexSize();
		Ok((s.x, s.y))
	}).unwrap());

	let _ = t.set("bounds",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let s = spr.getBounds();
		Ok((s.x, s.y, s.z, s.w))
	}).unwrap());

	let _ = t.set("setTextureRect",
	s.create_function(|s, x: (f32, f32, f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setTextureRect(glam::vec4(x.0, x.1, x.2, x.3));
		Ok(())
	}).unwrap());

	let _ = t.set("setAnimation",
	s.create_function(|s, x: String|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setAnimation(x);
		Ok(())
	}).unwrap());

	// let _ = t.set("loadAnimation",
	// s.create_function(|s, x: String|
	// {
	// 	*getSprite(s.globals().raw_get("ScriptID").unwrap()) = Sprite::animated(x);
	// 	Ok(())
	// }).unwrap());

	// let _ = t.set("loadImage",
	// s.create_function(|s, x: String|
	// {
	// 	*getSprite(s.globals().raw_get("ScriptID").unwrap()) = Sprite::image(x);
	// 	Ok(())
	// }).unwrap());

	// TODO Combine 'load' animations in one (use table { image = "", anim = "" })

	let _ = t.set("setColor",
	s.create_function(|s, x: (u8, u8, u8, u8)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setColor(x);
		Ok(())
	}).unwrap());

	let _ = t.set("resetAnimation",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.restart();
		Ok(())
	}).unwrap());

	let _ = s.globals().set("sprite", t);
}

pub fn text(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		Window::getCamera().draw(txt);
		Ok(())
	}).unwrap());

	let _ = t.set("size",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let d = txt.getDimensions();
		Ok((d.x, d.y))
	}).unwrap());

	let _ = t.set("bounds",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let d = txt.getBounds();
		Ok((d.x, d.y, d.z, d.w))
	}).unwrap());

	let _ = t.set("setString",
	s.create_function(|s, x: String|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.setString(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getString",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		Ok(txt.getString())
	}).unwrap());

	let _ = t.set("setColor",
	s.create_function(|s, x: (u8, u8, u8, u8)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.setColor(glam::vec4(
			x.0 as f32 / 255.0,
			x.1 as f32 / 255.0,
			x.2 as f32 / 255.0,
			x.3 as f32 / 255.0
		));
		Ok(())
	}).unwrap());

	let _ = t.set("getColor",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let c = txt.getColor();
		Ok((c.x, c.y, c.z, c.w))
	}).unwrap());

	let _ = s.globals().set("text", t);
}

pub fn window(script: &Lua)
{
	let table = script.create_table().unwrap();

	let _ = table.raw_set("launchServer",
	script.create_function(|_, _: ()|
	{
		Window::launchServer();
		Ok(())
	}).unwrap());

	let _ = table.raw_set("size",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getSize())
	}).unwrap());

	let _ = table.raw_set("screenSize",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getScreenSize())
	}).unwrap());
	
	let _ = table.raw_set("clearCache",
	script.create_function(|_, _: ()|
	{
		Window::clearCache();
		Ok(())
	}).unwrap());
	
	let _ = table.raw_set("resetDT",
	script.create_function(|_, _: ()|
	{
		Window::resetDT();
		Ok(())
	}).unwrap());

	let _ = table.raw_set("dt",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getDeltaTime())
	}).unwrap());

	let _ = table.raw_set("getNum",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().prog.get(&name)
			.unwrap_or(&Variable::default()).num)
	}).unwrap());

	let _ = table.raw_set("getStr",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().prog.get(&name)
			.unwrap_or(&Variable::default()).string.clone())
	}).unwrap());
	
	let _ = table.raw_set("setNum",
	script.create_function(|_, x: (String, f32)|
	{
		Window::getInstance().prog.insert(
			x.0,
			Variable { num: x.1, string: String::new() }
		);
		Ok(())
	}).unwrap());
	
	let _ = table.raw_set("setStr",
	script.create_function(|_, x: (String, String)|
	{
		Window::getInstance().prog.insert(
			x.0,
			Variable { num: 0.0, string: x.1 }
		);
		Ok(())
	}).unwrap());

	let _ = table.raw_set("mousePos",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getInstance().window.as_ref().unwrap().get_cursor_pos())
	}).unwrap());

	let _ = table.raw_set("setMousePos",
	script.create_function(|_, x: (f32, f32)|
	{
		Window::setMousePos(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = table.raw_set("showCursor",
	script.create_function(|_, x: bool|
	{
		Window::showCursor(x);
		Ok(())
	}).unwrap());
	
	let _ = table.raw_set("mousePressed",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().window.as_ref().unwrap()
			.get_mouse_button(Window::strToMB(name)) == glfw::Action::Press)
	}).unwrap());

	let _ = table.raw_set("mouseJustPressed",
	script.create_function(|_, name: String|
	{
		let e = Window::getInstance().mouseEvent;
		if e.is_none() { return Ok(false); }
		let e = e.unwrap();
		Ok(e.0 == Window::strToMB(name) && e.1 == glfw::Action::Press)
	}).unwrap());
	
	let _ = table.raw_set("keyPressed",
	script.create_function(|_, name: String|
	{
		Ok(Window::getInstance().window.as_ref().unwrap()
			.get_key(Window::strToKey(name)) == glfw::Action::Press)
	}).unwrap());
	
	let _ = table.raw_set("keyJustPressed",
	script.create_function(|_, name: String|
	{
		let e = Window::getInstance().keyEvent;
		if e.is_none() { return Ok(false); }
		let e = e.unwrap();
		Ok(
			e.0 == Window::strToKey(name) &&
			(e.1 == glfw::Action::Press || e.1 == glfw::Action::Repeat)
		)
	}).unwrap());
	
	let _ = table.raw_set("keyModPressed",
	script.create_function(|_, name: String|
	{
		let e = Window::getInstance().keyEvent;
		if e.is_none() { return Ok(false); }
		Ok(e.unwrap().2.intersects(Window::strToMod(name)))
	}).unwrap());

	let _ = table.raw_set("close",
	script.create_function(|_, _: ()|
	{
		Window::close(); Ok(())
	}).unwrap());
	
	let _ = table.raw_set("execute",
	script.create_function(|_, code: (String, String)|
	{
		if let Err(x) = getScript(code.0.clone())
			.load(code.1)
			.exec()
		{
			println!("{}: {x}", code.0);
		}
		Ok(())
	}).unwrap());

	let _ = table.set("loadUI",
	script.create_function(|_, path: String|
	{
		Window::getUI().requestLoad(path);
		Ok(())
	}).unwrap());

	let _ = table.set("uiScale",
	script.create_function(|_, _: ()|
	{
		let s1 = Window::getUI().getSize();
		let s2 = Window::getSize();
		Ok((s2.0 as f32 / s1.x, s2.1 as f32 / s1.y))
	}).unwrap());

	let _ = table.raw_set("input",
	script.create_function(|_, _: ()|
	{
		let x = Window::getInstance().inputEvent;
		if let Some(c) = x { Ok(c.to_string()) }
		else { Ok(String::new()) }
	}).unwrap());

	let _ = table.raw_set("clipboard",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getInstance().window.as_mut().unwrap()
			.get_clipboard_string().unwrap_or_default())
	}).unwrap());

	let _ = table.raw_set("setClipboard",
	script.create_function(|_, x: String|
	{
		Window::getInstance().window.as_mut().unwrap().set_clipboard_string(&x);
		Ok(())
	}).unwrap());

	let _ = table.raw_set("droppedFiles",
	script.create_function(|s, _: ()|
	{
		let t = s.create_table().unwrap();
		if let Some(f) = &Window::getInstance().dndEvent
		{
			for x in f { let _ = t.raw_push(x.as_str()); }
		}
		Ok(t)
	}).unwrap());

	let _ = table.raw_set("mouseWheel",
	script.create_function(|_, _: ()|
	{
		if let Some(f) = &Window::getInstance().scrollEvent { Ok(*f) }
		else { Ok(0.0) }
	}).unwrap());

	let _ = table.raw_set("isFocused",
	script.create_function(|_, _: ()|
	{
		Ok(Window::isFocused())
	}).unwrap());

	let _ = table.raw_set("isMaximized",
	script.create_function(|_, _: ()|
	{
		Ok(Window::isMaximized())
	}).unwrap());

	let _ = table.raw_set("clearColor",
	script.create_function(|_, x: (f32, f32, f32)|
	{
		unsafe
		{
			gl::ClearColor(x.0, x.1, x.2, 1.0);
		}
		Ok(())
	}).unwrap());

	let _ = script.globals().raw_set("window", table);
}

pub fn world(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.raw_set("name",
	script.create_function(|_, _: ()|
	{
		Ok(Window::getWorld().getName())
	}).unwrap());

	let _ = t.raw_set("load",
	script.create_function(|_, path: String|
	{
		Window::getWorld().load(path);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("reset",
	script.create_function(|_, _: ()|
	{
		*Window::getWorld() = World::init();
		Ok(())
	}).unwrap());

	let _ = t.raw_set("parse",
	script.create_function(|_, x: (String, String)|
	{
		Window::getWorld().parse(x.0, x.1);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("spawn",
	script.create_function(|_, data: (String, String, Table)|
	{
		let mut obj = json::object! {};
		for v in data.2.pairs::<String, mlua::Value>()
		{
			if let Err(_) = v { continue; }
			let (var, value) = v.unwrap();
			let _ = if value.is_integer() { obj.insert(&var, value.as_i32().unwrap()) }
			else if value.is_number() { obj.insert(&var, value.as_f32().unwrap()) }
			else if value.is_boolean() { obj.insert(&var, value.as_boolean().unwrap()) }
			else { obj.insert(&var, value.as_string_lossy().unwrap()) };
		}
		Window::getWorld().spawn(data.0, data.1, obj);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("kill",
	script.create_function(|_, x: String|
	{
		Window::getWorld().kill(x);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("reset",
	script.create_function(|_, _: ()|
	{
		*Window::getWorld() = World::init();
		Ok(())
	}).unwrap());
	
	let _ = script.globals().raw_set("world", t);
}

pub fn shaders(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.raw_set("bind",
	script.create_function(|_, name: String|
	{
		Window::getCamera().shaderUse(&name);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setBool",
	script.create_function(|_, x: (String, String, bool)|
	{
		Window::getCamera().shaderUse(&x.0);
		Window::getCamera().shaderBool(&x.1, x.2);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setInt",
	script.create_function(|_, x: (String, String, i32)|
	{
		Window::getCamera().shaderUse(&x.0);
		Window::getCamera().shaderInt(&x.1, x.2);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setVec2",
	script.create_function(|_, x: (String, String, f32, f32)|
	{
		Window::getCamera().shaderUse(&x.0);
		Window::getCamera().shaderVec2(&x.1, glam::vec2(x.2, x.3));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setVec3",
	script.create_function(|_, x: (String, String, f32, f32, f32)|
	{
		Window::getCamera().shaderUse(&x.0);
		Window::getCamera().shaderVec3(&x.1, glam::vec3(x.2, x.3, x.4));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setVec4",
	script.create_function(|_, x: (String, String, f32, f32, f32, f32)|
	{
		Window::getCamera().shaderUse(&x.0);
		Window::getCamera().shaderVec4(&x.1, glam::vec4(x.2, x.3, x.4, x.5));
		Ok(())
	}).unwrap());
	
	let _ = script.globals().raw_set("shaders", t);
}

pub fn profiler(script: &Lua)
{
    let t = script.create_table().unwrap();

    let _ = t.raw_set("restart",
    script.create_function(|_, _: ()|
    {
        Window::getProfiler().restart();
        Ok(())
    }).unwrap());

    let _ = t.raw_set("save",
    script.create_function(|_, name: String|
    {
        Ok(Window::getProfiler().save(name))
    }).unwrap());

    let _ = t.raw_set("get",
    script.create_function(|_, name: String|
    {
        Ok(Window::getProfiler().get(name))
    }).unwrap());

    let _ = script.globals().set("profiler", t);
}

pub fn mesh(s: &Lua)
{
    let t = s.create_table().unwrap();

	func(s, &t, "setTransform", |s, x: Table|
	{
		let ts = getEntity(s).getMesh().getTransformable();
		if let Ok(pos) = x.raw_get::<Table>("pos")
		{
			let mut p = ts.getPosition();
			p.x = pos.raw_get("x").unwrap_or(p.x);
			p.y = pos.raw_get("y").unwrap_or(p.y);
			p.z = pos.raw_get("z").unwrap_or(p.z);
			ts.setPosition(p);
		}
		if let Ok(angle) = x.raw_get::<Table>("angle")
		{
			ts.setRotation(glam::vec2(
				angle.raw_get("yaw").unwrap_or(0.0),
				angle.raw_get("pitch").unwrap_or(0.0)
			));
		}
		if let Ok(scale) = x.raw_get::<f32>("scale")
		{
			ts.setScale(scale);
		}
		Ok(())
	});

	let _ = t.set("load",
	s.create_function(|s, p: (String, usize)|
	{
		let mesh = getMesh(s.globals().raw_get("ScriptID").unwrap());
		let gltf = GLTF::load(p.0);
		*mesh = Mesh::fromGLTF(&gltf, p.1);
		Ok(())
	}).unwrap());

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		Window::getCamera().draw(
			getMesh(s.globals().raw_get("ScriptID").unwrap())
		);
		Ok(())
	}).unwrap());
	
    let _ = s.globals().set("mesh", t);
}

pub fn skeleton(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.set("setAnimation",
	script.create_function(|s, anim: String|
	{
		let sk = getSkeleton(
			s.globals().raw_get("ScriptID").unwrap()
		);
		sk.setAnimation(anim);
		Ok(())
	}).unwrap());

	let _ = t.set("load",
	script.create_function(|s, p: (String, usize)|
	{
		let sk = getSkeleton(
			s.globals().raw_get("ScriptID").unwrap()
		);
		let gltf = GLTF::load(p.0);
		*sk = Skeleton::fromGLTF(&gltf, p.1);
		Ok(())
	}).unwrap());

	let _ = t.set("update",
	script.create_function(|s, _: ()|
	{
		getSkeleton(s.globals().raw_get("ScriptID").unwrap()).update(
			Window::getCamera()
		);
		Ok(())
	}).unwrap());

	let _ = script.globals().set("skeleton", t);
}

pub fn network(s: &Lua)
{
	let t = s.create_table().unwrap();

	func(s, &t, "connect", |_, ip: String| Ok(Window::getNetwork().connect(ip)));
	func(s, &t, "disconnect", |_, _: ()| { Window::getNetwork().reset(); Ok(()) });
	func(s, &t, "isReady", |_, _: ()| Ok(Window::getNetwork().isReady()));
	func(s, &t, "id", |_, _: ()| Ok(Window::getNetwork().getID()));

	func(s, &t, "send", |_, data: Table|
	{
		let _ = data.len();
		Ok(())
	});
	
	func(s, &t, "setup", |_, data: Table|
	{
		Window::getNetwork().setup(
			data.raw_get("tickRate").unwrap_or(10),
			data.raw_get("id").unwrap_or(0),
			data.raw_get("port").unwrap_or(26225)
		);
		Ok(())
	});

	func(s, &t, "hasMessage", |_, topic: String|
	{
		Ok(Window::getNetwork().hasMessage(topic))
	});

	func(s, &t, "getMessage", |s, topic: String|
	{
		let t = s.create_table().unwrap();
		let data = Window::getNetwork().getMessage(topic.clone());
		match topic.as_str()
		{
			"setup" =>
			{
				let _ = t.raw_set("tickRate", data["tickRate"].as_u8().unwrap());
				let _ = t.raw_set("port", data["port"].as_u16().unwrap());
				let _ = t.raw_set("id", data["id"].as_u8().unwrap());
			}
			x => { println!("Unknown topic: {x}"); }
		}
		Ok(t)
	});

	func(s, &t, "setState", |_, data: (f32, f32, f32, f32, f32)|
	{
		Window::getNetwork().setState(
			glam::vec3(data.0, data.1, data.2),
			glam::vec2(data.3, data.4)
		);
		Ok(())
	});

	func(s, &t, "getState", |_, id: u8|
	{
		let s = Window::getNetwork().getState(id);
		Ok((s.0.x, s.0.y, s.0.z, s.1.x, s.1.y))
	});

	let _ = s.globals().set("network", t);
}

pub fn camera(s: &Lua)
{
	let t = s.create_table().unwrap();

	func(s, &t, "setFOV", |_, x: f32|
	{
		Window::getCamera().setFOV(x);
		Ok(())
	});

	func(s, &t, "setScale", |_, x: i32|
	{
		Window::getCamera().setScaler(x);
		Ok(())
	});

	func(s, &t, "setMode", |_, data: Table|
	{
		let ts = Window::getCamera().getTransformable();
		if let Ok(_) = data.raw_get::<Table>("firstPerson")
		{
			ts.setRotationMode(super::Transformable::RotationMode::LookAtFP);
		}
		if let Ok(tp) = data.raw_get::<Table>("thirdPerson")
		{
			ts.setRotationMode(super::Transformable::RotationMode::LookAtTP(
				tp.raw_get::<f32>("distance").unwrap_or(2.0)
			));
		}
		Ok(())
	});

	func(s, &t, "setTransform", |_, x: Table|
	{
		let ts = Window::getCamera().getTransformable();
		if let Ok(pos) = x.raw_get::<Table>("pos")
		{
			let p = ts.getPosition();
			ts.setPosition(glam::vec3(
				pos.raw_get("x").unwrap_or(p.x),
				pos.raw_get("y").unwrap_or(p.y),
				pos.raw_get("z").unwrap_or(p.z)
			));
		}
		if let Ok(angle) = x.raw_get::<Table>("angle")
		{
			ts.setRotation(glam::vec2(
				angle.raw_get("yaw").unwrap_or(0.0),
				angle.raw_get("pitch").unwrap_or(0.0)
			));
		}
		Ok(())
	});

	func(s, &t, "addTransform", |_, x: Table|
	{
		let ts = Window::getCamera().getTransformable();
		if let Ok(pos) = x.raw_get::<Table>("translation")
		{
			ts.translate(glam::vec3(
				pos.raw_get("x").unwrap_or(0.0),
				pos.raw_get("y").unwrap_or(0.0),
				pos.raw_get("z").unwrap_or(0.0)
			));
		}
		if let Ok(pos) = x.raw_get::<Table>("movement")
		{
			let d = ts.getFront();
			let dx = pos.raw_get("x").unwrap_or(0.0);
			let dy = pos.raw_get("y").unwrap_or(0.0);
			let dz = pos.raw_get("z").unwrap_or(0.0);
			ts.translate(glam::vec3(
				d.x * dz - d.y * dx,
				dy,
				d.y * dz + d.x * dx
			));
		}
		if let Ok(pos) = x.raw_get::<Table>("fly")
		{
			let d = ts.getDirection();
			let dx = pos.raw_get("x").unwrap_or(0.0);
			let dy = pos.raw_get("y").unwrap_or(0.0);
			let dz = pos.raw_get("z").unwrap_or(0.0);
			ts.translate(glam::vec3(
				d.x * dz - d.z * dx,
				d.y * dz + dy,
				d.z * dz + d.x * dx
			));
		}
		if let Ok(angle) = x.raw_get::<Table>("angle")
		{
			ts.rotate(glam::vec2(
				angle.raw_get("yaw").unwrap_or(0.0),
				angle.raw_get("pitch").unwrap_or(0.0)
			));
		}
		Ok(())
	});

	func(s, &t, "getTransform", |s, x: Table|
	{
		let out = s.create_table().unwrap();
		let ts = Window::getCamera().getTransformable();
		for i in x.pairs::<u8, String>()
		{
			if let Ok(i) = i
			{
				if i.1 == "pos"
				{
					let p = s.create_table().unwrap();
					let pos = ts.getPosition();
					let _ = p.raw_set("x", pos.x);
					let _ = p.raw_set("y", pos.y);
					let _ = p.raw_set("z", pos.z);
					let _ = out.raw_set("pos", p);
				}
				if i.1 == "angle"
				{
					let a = s.create_table().unwrap();
					let angle = ts.getRotation();
					let _ = a.raw_set("yaw", angle.x);
					let _ = a.raw_set("pitch", angle.y);
					let _ = out.raw_set("angle", a);
				}
			}
		}
		Ok(out)
	});

	let _ = s.globals().raw_set("camera", t);
}