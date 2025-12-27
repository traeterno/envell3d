use mlua::{Lua, Table};

use crate::ae3d::{glTF::GLTF, Mesh::Mesh, Skeleton::Skeleton, Transformable::Transformable2D};
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
	Window::getWorld().getEntity(
		id.split("_").nth(1).unwrap().to_string()
	)
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

	let _ = t.set("loadAnimation",
	s.create_function(|s, x: String|
	{
		*getSprite(s.globals().raw_get("ScriptID").unwrap()) = Sprite::animated(x);
		Ok(())
	}).unwrap());

	let _ = t.set("loadImage",
	s.create_function(|s, x: String|
	{
		*getSprite(s.globals().raw_get("ScriptID").unwrap()) = Sprite::image(x);
		Ok(())
	}).unwrap());

	let _ = t.set("setColor",
	s.create_function(|s, x: (u8, u8, u8, u8)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.setColor(x);
		Ok(())
	}).unwrap());

	let _ = t.set("setPosition",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setPosition(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("translate",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().translate(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getPosition",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getPosition();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setOrigin",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setOrigin(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getOrigin",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getOrigin();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setScale",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setScale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("scale",
	s.create_function(|s, x: (f32, f32)|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().scale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getScale",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getScale();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setRotation",
	s.create_function(|s, x: f32|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().setRotation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("rotate",
	s.create_function(|s, x: f32|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.getTransformable().rotate(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getRotation",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let x = spr.getTransformable().getRotation();
		Ok(x)
	}).unwrap());

	let _ = t.set("applyModel",
	s.create_function(|s, shader: String|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		Window::getCamera().shaderUse(&shader);
		Window::getCamera().shaderMat4("model", spr.getTransformable().getMatrix());
		Ok(())
	}).unwrap());

	let _ = t.set("getCurrentFrame",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		let f = spr.getCurrentFrame();
		Ok((f.x, f.y, f.z, f.w))
	}).unwrap());

	let _ = t.set("bindTexture",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		unsafe
		{
			gl::BindTexture(gl::TEXTURE_2D, spr.getTexture());
		}
		Ok(())
	}).unwrap());

	let _ = t.set("tickAnimation",
	s.create_function(|s, _: ()|
	{
		let spr = getSprite(s.globals().raw_get("ScriptID").unwrap());
		spr.update();
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

	let _ = t.set("setPosition",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setPosition(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("translate",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().translate(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getPosition",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getPosition();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setOrigin",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setOrigin(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getOrigin",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getOrigin();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setScale",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setScale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("scale",
	s.create_function(|s, x: (f32, f32)|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().scale(glam::vec2(x.0, x.1));
		Ok(())
	}).unwrap());

	let _ = t.set("getScale",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getScale();
		Ok((x.x, x.y))
	}).unwrap());

	let _ = t.set("setRotation",
	s.create_function(|s, x: f32|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().setRotation(x);
		Ok(())
	}).unwrap());

	let _ = t.set("rotate",
	s.create_function(|s, x: f32|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		txt.getTransformable().rotate(x);
		Ok(())
	}).unwrap());

	let _ = t.set("getRotation",
	s.create_function(|s, _: ()|
	{
		let txt = getText(s.globals().raw_get("ScriptID").unwrap());
		let x = txt.getTransformable().getRotation();
		Ok(x)
	}).unwrap());

	let _ = s.globals().set("text", t);
}

pub fn network(s: &Lua)
{
	let t = s.create_table().unwrap();

	let _ = t.raw_set("connect",
	s.create_function(|_, ip: String|
	{
		Ok(Window::getNetwork().connect(ip))
	}).unwrap());

	let _ = t.raw_set("disconnect",
	s.create_function(|_, _: ()|
	{
		Window::getNetwork().reset();
		Ok(())
	}).unwrap());

	let _ = t.raw_set("send",
	s.create_function(|_, data: mlua::Table|
	{
		let mut buf = vec![];
		for x in data.pairs::<u8, mlua::Value>()
		{
			if let Ok((_, v)) = x
			{
				if v.is_number()
				{
					buf.append(&mut v.as_f32().unwrap().to_be_bytes().to_vec());
				}
			}
		}
		Window::getNetwork().send(buf);
		Ok(())
	}).unwrap());

	let _ = s.globals().set("network", t);
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

	let _ = table.set("uiSize",
	script.create_function(|_, _: ()|
	{
		let s = Window::getUI().getSize();
		Ok((s.x, s.y))
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

	// let _ = table.raw_set("screenToWorld",
	// script.create_function(|_, x: (f32, f32)|
	// {
	// 	let a = Window::getCamera().getTransformable().getPosition();
	// 	let s1 = Window::getCamera().getSize();
	// 	let s2 = Window::getSize();
	// 	let s3 = glam::vec2(s1.x / s2.0 as f32, s1.y / s2.1 as f32);
	// 	let p = -a - s1 / 2.0 + glam::vec2(x.0 * s3.x, x.1 * s3.y);
	// 	Ok((p.x, p.y))
	// }).unwrap());

	// let _ = table.raw_set("worldToScreen",
	// script.create_function(|_, x: (f32, f32, f32)|
	// {
	// 	let t = Window::getCamera().getTransformable();
	// 	let p = t.getMatrix().transform_point3(glam::vec3(x.0, x.1, x.2));
	// 	let s1 = Window::getCamera().getSize();
	// 	let s2 = Window::getSize();
	// 	let s3 = glam::vec2(s2.0 as f32 / s1.x, s2.1 as f32 / s1.y);
	// 	Ok((p.x * s3.x, p.y * s3.y))
	// }).unwrap());

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

	let _ = t.raw_set("createTrigger",
	script.create_function(|_, x: (String, String, f32, f32, f32, f32)|
	{
		Window::getWorld().createTrigger(
			x.0, x.1,
			glam::vec4(x.2, x.3, x.4, x.5
		));
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("modifyTrigger",
	script.create_function(|_, x: (String, f32, f32, f32, f32)|
	{
		Window::getWorld().modifyTrigger(
			x.0, glam::vec4(x.1, x.2, x.3, x.4)
		);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("getTriggers",
	script.create_function(|s, _: ()|
	{
		let t = s.create_table().unwrap();
		for (id, hb) in Window::getWorld().getTriggers()
		{
			let h = s.create_table().unwrap();
			let _ = h.raw_set("name", hb.0.as_str());
			let _ = h.raw_set("x", hb.1.x);
			let _ = h.raw_set("y", hb.1.y);
			let _ = h.raw_set("w", hb.1.z);
			let _ = h.raw_set("h", hb.1.w);
			let _ = t.raw_set(id.clone(), h);
		}
		Ok(t)
	}).unwrap());

	let _ = t.raw_set("setNum",
	script.create_function(|_, data: (String, f32)|
	{
		Window::getWorld().getProgrammable().insert(
			data.0,
			Variable { num: data.1, string: String::new() }
		);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("setStr",
	script.create_function(|_, data: (String, String)|
	{
		Window::getWorld().getProgrammable().insert(
			data.0,
			Variable { num: 0.0, string: data.1 }
		);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("getNum",
	script.create_function(|_, data: String|
	{
		let v = Variable::default();
		Ok(Window::getWorld().getProgrammable().get(&data).unwrap_or(&v).num)
	}).unwrap());
	
	let _ = t.raw_set("getStr",
	script.create_function(|_, data: String|
	{
		let v = Variable::default();
		Ok(Window::getWorld().getProgrammable().get(&data).unwrap_or(&v).string.clone())
	}).unwrap());
	
	let _ = t.raw_set("setupCamera",
	script.create_function(|_, fov: f32|
	{
		Window::getCamera().setup(Window::getSize(), fov);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("setCamPos",
	script.create_function(|_, x: (f32, f32, f32)|
	{
		Window::getCamera().getTransformable().setPosition(
			glam::vec3(x.0, x.1, x.2)
		);
		Ok(())
	}).unwrap());
	
	let _ = t.raw_set("getCamPos",
	script.create_function(|_, _: ()|
	{
		let p = Window::getCamera().getTransformable().getPosition();
		Ok((p.x, p.y, p.z))
	}).unwrap());
	
	let _ = t.raw_set("camLookAt",
	script.create_function(|_, p: (f32, f32, f32)|
	{
		Window::getCamera().lookAt(glam::vec3(p.0, p.1, p.2));
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

pub fn shapes2D(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.raw_set("rect",
	script.create_function(|_, args: mlua::Table|
	{
		let x = args.raw_get::<f32>("x").unwrap_or(0.0);
		let y = args.raw_get::<f32>("y").unwrap_or(0.0);
		let ox = args.raw_get::<f32>("ox").unwrap_or(0.0);
		let oy = args.raw_get::<f32>("oy").unwrap_or(0.0);
		let w = args.raw_get::<f32>("w").unwrap_or(1.0);
		let h = args.raw_get::<f32>("h").unwrap_or(1.0);
		let angle = args.raw_get::<f32>("angle").unwrap_or(0.0);
		let r = args.raw_get::<f32>("r").unwrap_or(0.0) / 255.0;
		let g = args.raw_get::<f32>("g").unwrap_or(0.0) / 255.0;
		let b = args.raw_get::<f32>("b").unwrap_or(0.0) / 255.0;
		let a = args.raw_get::<f32>("a").unwrap_or(255.0) / 255.0;
		Window::getCamera().drawRect2D(
			Transformable2D::quick(
				glam::vec2(x, y), angle,
				glam::vec2(w, h),
				glam::vec2(ox, oy)
			), glam::vec4(r, g, b, a)
		);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("custom",
	script.create_function(|_, _: ()|
	{
		Window::getCamera().genericVAO();
		unsafe { gl::DrawArrays(gl::QUADS, 0, 4); }
		Ok(())
	}).unwrap());

	let _ = script.globals().raw_set("shapes2D", t);
}

pub fn shapes3D(script: &Lua)
{
	let t = script.create_table().unwrap();

	let _ = t.raw_set("rect",
	script.create_function(|_, args: mlua::Table|
	{
		// let x = args.raw_get::<f32>("x").unwrap_or(0.0);
		// let y = args.raw_get::<f32>("y").unwrap_or(0.0);
		// let z = args.raw_get::<f32>("z").unwrap_or(0.0);
		// let ox = args.raw_get::<f32>("ox").unwrap_or(0.0);
		// let oy = args.raw_get::<f32>("oy").unwrap_or(0.0);
		// let oz = args.raw_get::<f32>("oz").unwrap_or(0.0);
		// let w = args.raw_get::<f32>("w").unwrap_or(1.0);
		// let h = args.raw_get::<f32>("h").unwrap_or(1.0);
		// let d = args.raw_get::<f32>("d").unwrap_or(1.0);
		// // let angle = args.raw_get::<f32>("angle").unwrap_or(0.0);
		let r = args.raw_get::<f32>("r").unwrap_or(0.0) / 255.0;
		let g = args.raw_get::<f32>("g").unwrap_or(0.0) / 255.0;
		let b = args.raw_get::<f32>("b").unwrap_or(0.0) / 255.0;
		let a = args.raw_get::<f32>("a").unwrap_or(255.0) / 255.0;
		// Window::getCamera().drawRect3D(
		// 	Transformable3D::quick(
		// 		glam::vec2(x, y), angle,
		// 		glam::vec2(w, h),
		// 		glam::vec2(ox, oy)
		// 	), glam::vec4(r, g, b, a)
		// );
		Window::getCamera().drawRect3D(glam::Mat4::IDENTITY,
			glam::vec4(r, g, b, a)
		);
		Ok(())
	}).unwrap());

	let _ = t.raw_set("line",
	script.create_function(|_, args: mlua::Table|
	{
		Window::getCamera().drawLine3D(
			glam::vec3(
				args.raw_get::<f32>("x1").unwrap_or(0.0),
				args.raw_get::<f32>("y1").unwrap_or(0.0),
				args.raw_get::<f32>("z1").unwrap_or(0.0)
			),
			glam::vec3(
				args.raw_get::<f32>("x2").unwrap_or(0.0),
				args.raw_get::<f32>("y2").unwrap_or(0.0),
				args.raw_get::<f32>("z2").unwrap_or(0.0)
			),
			glam::vec4(
				args.raw_get::<f32>("r").unwrap_or(0.0) / 255.0,
				args.raw_get::<f32>("g").unwrap_or(0.0) / 255.0,
				args.raw_get::<f32>("b").unwrap_or(0.0) / 255.0,
				args.raw_get::<f32>("a").unwrap_or(0.0) / 255.0
			)
		);
		Ok(())
	}).unwrap());

	// let _ = t.raw_set("custom",
	// script.create_function(|_, _: ()|
	// {
	// 	Window::getCamera().genericVAO();
	// 	unsafe { gl::DrawArrays(gl::QUADS, 0, 4); }
	// 	Ok(())
	// }).unwrap());

	let _ = script.globals().raw_set("shapes3D", t);
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

pub fn mesh(script: &Lua)
{
    let t = script.create_table().unwrap();

	let _ = t.set("setPosition",
	script.create_function(|s, pos: (f32, f32, f32)|
	{
		let mesh = getMesh(s.globals().raw_get("ScriptID").unwrap());
		mesh.getTransformable().setPosition(glam::vec3(pos.0, pos.1, pos.2));
		Ok(())
	}).unwrap());

	let _ = t.set("setRotation",
	script.create_function(|s, angle: (f32, f32, f32)|
	{
		let mesh = getMesh(s.globals().raw_get("ScriptID").unwrap());
		mesh.getTransformable().setRotation(glam::vec3(angle.0, angle.1, angle.2));
		Ok(())
	}).unwrap());

	let _ = t.set("load",
	script.create_function(|s, p: (String, usize)|
	{
		let mesh = getMesh(s.globals().raw_get("ScriptID").unwrap());
		let gltf = GLTF::load(p.0);
		*mesh = Mesh::fromGLTF(&gltf, p.1);
		Ok(())
	}).unwrap());

	let _ = t.set("draw",
	script.create_function(|s, _: ()|
	{
		Window::getCamera().draw(
			getMesh(s.globals().raw_get("ScriptID").unwrap())
		);
		Ok(())
	}).unwrap());
	
    let _ = script.globals().set("mesh", t);
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

	let _ = t.set("draw",
	script.create_function(|s, _: ()|
	{
		Window::getCamera().draw(
			getSkeleton(s.globals().raw_get("ScriptID").unwrap())
		);
		Ok(())
	}).unwrap());

	let _ = script.globals().set("skeleton", t);
}