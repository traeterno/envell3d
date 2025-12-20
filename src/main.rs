#![allow(non_snake_case, static_mut_refs, dead_code)]
// #![windows_subsystem = "windows"]
mod ae3d;
mod server;

use ae3d::Window::Window;

use crate::ae3d::{glTF, Skeleton::Skeleton};

fn main()
{
	Window::init("res/global/config.json");
	Window::resetDT();

	let gltf = glTF::GLTF::load("res/objects/hero.gltf".to_string());

	let mut s = Skeleton::fromGLTF(&gltf, 0, 0);
	s.setAnimation("idle".to_string());

	let mut toggle = true;
	
	let cam = Window::getCamera();
	cam.toggleTransform(true);
	while Window::isOpen()
	{
		Window::update();

		if let Some((k, a, _)) = Window::getInstance().keyEvent
		{
			if k == glfw::Key::E && a == glfw::Action::Press
			{
				toggle = !toggle;
				s.setAnimation(if toggle { "idle".to_string() } else { String::new() });
			}
		}
		
		cam.clear();
		cam.toggleTransform(true);
		cam.draw(&mut s);
		Window::display();

		// Window::render();
	}
}
