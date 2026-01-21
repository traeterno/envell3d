use mlua::{Lua, Table};

use crate::ae3d::UI;
use crate::ae3d::{glTF::GLTF, Mesh::Mesh, Skeleton::Skeleton};
use crate::ae3d::{Entity::Entity, Programmable::Variable, World::World};

use super::Window::Window;

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

fn getEntity(s: &Lua) -> &'static mut Entity
{
	let id: String = s.globals().get("ScriptID").unwrap();
	Window::getWorld().getEntity(id.split("_").nth(1).unwrap().to_string())
}

fn getUIobj(s: &Lua) -> &'static mut UI::Object
{
	let id: String = s.globals().get("ScriptID").unwrap();
	Window::getUI().getObject(id.split("_").nth(1).unwrap().to_string())
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

	// TODO rewrite functions to new style

	let _ = s.globals().set("sprite", t);
}

pub fn text(s: &Lua)
{
	let t = s.create_table().unwrap();

	func(s, &t, "draw", |s, _: ()|
	{
		let txt = getUIobj(s).getText();
		Window::getCamera().draw(txt);
		Ok(())
	});

	func(s, &t, "size", |s, _: ()|
	{
		let txt = getUIobj(s).getText();
		let d = txt.getDimensions();
		Ok((d.x, d.y))
	});

	func(s, &t, "setTransform", |s, data: Table|
	{
		let txt = getUIobj(s).getText();
		let ts = txt.getTransformable();
		if let Ok(pos) = data.raw_get::<Table>("pos")
		{
			ts.setPosition(glam::vec2(
				pos.raw_get("x").unwrap_or(0.0),
				pos.raw_get("y").unwrap_or(0.0)
			));
		}
		if let Ok(angle) = data.raw_get("angle")
		{
			ts.setRotation(angle);
		}
		if let Ok(scale) = data.raw_get::<Table>("scale")
		{
			ts.setScale(glam::vec2(
				scale.raw_get("x").unwrap_or(1.0),
				scale.raw_get("y").unwrap_or(1.0)
			))
		}
		if let Ok(origin) = data.raw_get::<Table>("origin")
		{
			ts.setOrigin(glam::vec2(
				origin.raw_get("x").unwrap_or(0.0),
				origin.raw_get("y").unwrap_or(0.0)
			));
		}
		Ok(())
	});

	func(s, &t, "getTransform", |s, data: Table|
	{
		let t = s.create_table().unwrap();
		let ts = getUIobj(s).getText().getTransformable();
		for x in data.pairs::<u8, String>()
		{
			if let Ok((_, var)) = x
			{
				if var == "pos"
				{
					let p = s.create_table().unwrap();
					let pos = ts.getPosition();
					let _ = p.raw_set("x", pos.x);
					let _ = p.raw_set("y", pos.y);
					let _ = t.raw_set("pos", p);
				}
				if var == "scale"
				{
					let s = s.create_table().unwrap();
					let scale = ts.getScale();
					let _ = s.raw_set("x", scale.x);
					let _ = s.raw_set("y", scale.y);
					let _ = t.raw_set("scale", s);
				}
				if var == "origin"
				{
					let o = s.create_table().unwrap();
					let origin = ts.getOrigin();
					let _ = o.raw_set("x", origin.x);
					let _ = o.raw_set("y", origin.y);
					let _ = t.raw_set("origin", o);
				}
				if var == "angle"
				{
					let _ = t.raw_set("angle", ts.getRotation());
				}
			}
		}
		Ok(t)
	});

	let _ = t.set("bounds",
	s.create_function(|s, _: ()|
	{
		let txt = getUIobj(s).getText();
		let d = txt.getBounds();
		Ok((d.x, d.y, d.z, d.w))
	}).unwrap());

	let _ = t.set("setString",
	s.create_function(|s, x: String|
	{
		let txt = getUIobj(s).getText();
		txt.setString(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getString",
	s.create_function(|s, _: ()|
	{
		let txt = getUIobj(s).getText();
		Ok(txt.getString())
	}).unwrap());

	let _ = t.set("setColor",
	s.create_function(|s, x: (u8, u8, u8, u8)|
	{
		let txt = getUIobj(s).getText();
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
		let txt = getUIobj(s).getText();
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

	func(script, &table, "resetCursor", |_, _: ()|
	{
		let m = Window::getInstance()
			.window.as_ref().unwrap().get_cursor_pos();
		let s = Window::getSize();
		let c = (s.0 as f32 * 0.5, s.1 as f32 * 0.5);
		Window::setMousePos(glam::vec2(c.0, c.1));
		Ok((m.0 as f32 - c.0, m.1 as f32 - c.1))
	});

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
			ts.getOrientation().set(glam::vec3(
				angle.raw_get("yaw").unwrap_or(0.0),
				angle.raw_get("pitch").unwrap_or(0.0),
				angle.raw_get("roll").unwrap_or(0.0)
			));
		}
		if let Ok(scale) = x.raw_get::<f32>("scale")
		{
			ts.setScale(scale);
		}
		Ok(())
	});

	func(s, &t, "addTransform", |s, x: Table|
	{
		let ts = getEntity(s).getMesh().getTransformable();
		if let Ok(pos) = x.raw_get::<Table>("pos")
		{
			let mut p = ts.getPosition();
			p.x = pos.raw_get("x").unwrap_or(p.x);
			p.y = pos.raw_get("y").unwrap_or(p.y);
			p.z = pos.raw_get("z").unwrap_or(p.z);
			ts.translate(p);
		}
		if let Ok(angle) = x.raw_get::<Table>("angle")
		{
			ts.getOrientation().add(glam::vec3(
				angle.raw_get("yaw").unwrap_or(0.0),
				angle.raw_get("pitch").unwrap_or(0.0),
				angle.raw_get("roll").unwrap_or(0.0)
			));
		}
		if let Ok(scale) = x.raw_get::<f32>("scale")
		{
			ts.scale(scale);
		}
		Ok(())
	});

	func(s, &t, "getTransform", |s, data: Table|
	{
		let ts = getEntity(s).getMesh().getTransformable();
		let t = s.create_table().unwrap();
		for x in data.pairs::<u8, String>()
		{
			if let Ok((_, var)) = x
			{
				if var == "pos"
				{
					let p = s.create_table().unwrap();
					let pos = ts.getPosition();
					let _ = p.raw_set("x", pos.x);
					let _ = p.raw_set("y", pos.y);
					let _ = p.raw_set("z", pos.z);
					let _ = t.raw_set("pos", p);
				}
				if var == "angle"
				{
					let e = s.create_table().unwrap();
					let angle = ts.getOrientation().getAngle();
					let _ = e.raw_set("yaw", angle.x);
					let _ = e.raw_set("pitch", angle.y);
					let _ = e.raw_set("roll", angle.z);
					let _ = t.raw_set("angle", e);
				}
				if var == "scale"
				{
					let _ = t.raw_set("scale", ts.getScale());
				}
			}
		}
		Ok(t)
	});

	let _ = t.set("load",
	s.create_function(|s, p: (String, usize)|
	{
		let mesh = getEntity(s).getMesh();
		let gltf = GLTF::load(p.0);
		*mesh = Mesh::fromGLTF(&gltf, p.1);
		Ok(())
	}).unwrap());

	let _ = t.set("draw",
	s.create_function(|s, _: ()|
	{
		Window::getCamera().draw(getEntity(s).getMesh());
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
		let sk = getEntity(s).getSkeleton();
		sk.setAnimation(anim);
		Ok(())
	}).unwrap());

	let _ = t.set("load",
	script.create_function(|s, p: (String, usize)|
	{
		let sk = getEntity(s).getSkeleton();
		let gltf = GLTF::load(p.0);
		*sk = Skeleton::fromGLTF(&gltf, p.1);
		Ok(())
	}).unwrap());

	let _ = t.set("update",
	script.create_function(|s, _: ()|
	{
		getEntity(s).getSkeleton().update(
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
	func(s, &t, "isActive", |_, _: ()| Ok(Window::getNetwork().isActive()));
	func(s, &t, "id", |_, _: ()| Ok(Window::getNetwork().getID()));
	func(s, &t, "discovered", |_, _: ()| { Ok(Window::getNetwork().discoveredIP()) });

	func(s, &t, "search", |_, _: ()|
	{
		let _ = std::thread::Builder::new()
			.name(String::from("Server Search"))
			.spawn(|| crate::ae3d::Network::search());
		Ok(())
	});


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

	func(s, &t, "setDistance", |_, x: f32|
	{
		Window::getCamera().setDistance(x);
		Ok(())
	});

	func(s, &t, "setTransform", |_, x: Table|
	{
		let c = Window::getCamera();
		if let Ok(pos) = x.raw_get::<Table>("pos")
		{
			c.setPosition(glam::vec3(
				pos.raw_get("x").unwrap_or(0.0),
				pos.raw_get("y").unwrap_or(0.0),
				pos.raw_get("z").unwrap_or(0.0)
			));
		}
		if let Ok(angle) = x.raw_get::<Table>("angle")
		{
			c.setRotation(glam::vec3(
				angle.raw_get("yaw").unwrap_or(0.0),
				angle.raw_get("pitch").unwrap_or(0.0),
				angle.raw_get("roll").unwrap_or(0.0)
			));
		}
		Ok(())
	});

	func(s, &t, "addTransform", |_, x: Table|
	{
		let c = Window::getCamera();
		if let Ok(pos) = x.raw_get::<Table>("translation")
		{
			c.translate(glam::vec3(
				pos.raw_get("x").unwrap_or(0.0),
				pos.raw_get("y").unwrap_or(0.0),
				pos.raw_get("z").unwrap_or(0.0)
			));
		}
		if let Ok(pos) = x.raw_get::<Table>("fly")
		{
			let d = c.getOrientation().getDirection();
			let dx = pos.raw_get("x").unwrap_or(0.0);
			let dy = pos.raw_get("y").unwrap_or(0.0);
			let dz = pos.raw_get("z").unwrap_or(0.0);
			c.translate(glam::vec3(
				d.x * dz - d.z * dx,
				d.y * dz + dy,
				d.z * dz + d.x * dx
			));
		}
		if let Ok(angle) = x.raw_get::<Table>("angle")
		{
			c.rotate(glam::vec3(
				angle.raw_get("yaw").unwrap_or(0.0),
				angle.raw_get("pitch").unwrap_or(0.0),
				angle.raw_get("roll").unwrap_or(0.0)
			));
		}
		Ok(())
	});

	func(s, &t, "getTransform", |s, x: Table|
	{
		let out = s.create_table().unwrap();
		let c = Window::getCamera();
		for i in x.pairs::<u8, String>()
		{
			if let Ok(i) = i
			{
				if i.1 == "pos"
				{
					let p = s.create_table().unwrap();
					let pos = c.getPosition();
					let _ = p.raw_set("x", pos.x);
					let _ = p.raw_set("y", pos.y);
					let _ = p.raw_set("z", pos.z);
					let _ = out.raw_set("pos", p);
				}
				if i.1 == "angle"
				{
					let a = s.create_table().unwrap();
					let angle = c.getOrientation().getAngle();
					let _ = a.raw_set("yaw", angle.x);
					let _ = a.raw_set("pitch", angle.y);
					let _ = a.raw_set("roll", angle.z);
					let _ = out.raw_set("angle", a);
				}
				if i.1 == "direction"
				{
					let d = s.create_table().unwrap();
					let dir = c.getOrientation().getDirection();
					let _ = d.raw_set("x", dir.x);
					let _ = d.raw_set("y", dir.y);
					let _ = d.raw_set("z", dir.z);
					let _ = out.raw_set("direction", d);
				}
			}
		}
		Ok(out)
	});

	let _ = s.globals().raw_set("camera", t);
}

// TODO
pub fn math(s: &Lua)
{
	let t = s.create_table().unwrap();
	
	func(s, &t, "clamp", |_, x: (f32, f32, f32)|
	{
		Ok(x.0.clamp(x.1, x.2))
	});

	func(s, &t, "rectContains", |_,  x: (Table, Table)|
	{
		let p = glam::vec2(
			x.0.raw_get("x").unwrap_or(0.0),
			x.0.raw_get("y").unwrap_or(0.0)
		);
		let r = glam::vec4(
			x.1.raw_get("x").unwrap_or(0.0),
			x.1.raw_get("y").unwrap_or(0.0),
			x.1.raw_get("w").unwrap_or(0.0),
			x.1.raw_get("h").unwrap_or(0.0)
		);
		Ok(
			p.x == p.x.clamp(r.x, r.x + r.z) &&
			p.y == p.y.clamp(r.y, r.y + r.w)
		)
	});

	func(s, &t, "rectIntersects", |_, i: (Table, Table)|
	{
		let r1 = glam::vec4(
			i.0.raw_get("x").unwrap_or(0.0),
			i.0.raw_get("y").unwrap_or(0.0),
			i.0.raw_get("z").unwrap_or(0.0),
			i.0.raw_get("w").unwrap_or(0.0),
		);
		let r2 = glam::vec4(
			i.1.raw_get("x").unwrap_or(0.0),
			i.1.raw_get("y").unwrap_or(0.0),
			i.1.raw_get("z").unwrap_or(0.0),
			i.1.raw_get("w").unwrap_or(0.0),
		);
		let il = r1.x.max(r2.x);
		let it = r1.y.max(r2.y);
		let ir = (r1.x + r1.z).min(r2.x + r2.z);
		let ib = (r1.y + r1.w).min(r2.y + r2.w);
		Ok((il < ir) && (it < ib))
	});

	func(s, &t, "lerp", |_, x: (f32, f32, f32)|
	{
		let a = x.0; let b = x.1; let t = x.2;
		Ok(a * (1.0 - t) + b * t)
	});

	func(s, &t, "cubicIn", |_, x: f32|
	{
		Ok(x.powi(3))
	});

	func(s, &t, "cubicOut", |_, x: f32|
	{
		Ok(1.0 - (1.0 - x).powi(3))
	});

	func(s, &t, "cubicInOut", |_, x: f32|
	{
		Ok(
			if x < 0.5 { 4.0 * x.powi(3) }
			else { 1.0 - (-2.0 * x + 2.0).powi(3) * 0.5 }
		)
	});

	func(s, &t, "sineIn", |_, x: f32|
	{
		Ok(1.0 - (x * std::f32::consts::PI).cos() * 0.5)
	});

	func(s, &t, "sineOut", |_, x: f32|
	{
		Ok((x * std::f32::consts::PI * 0.5).sin())
	});

	func(s, &t, "sineInOut", |_, x: f32|
	{
		Ok(-((x * std::f32::consts::PI).cos() - 1.0) * 0.5)
	});

	func(s, &t, "round", |_, x: (f32, u8)|
	{
		let factor = 10.0_f32.powi(x.1 as i32);
		Ok((x.0 * factor).round() / factor)
	});

	let _ = s.globals().raw_set("aemath", t);
}