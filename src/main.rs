#![allow(non_snake_case, static_mut_refs, dead_code)]
// #![windows_subsystem = "windows"]
mod ae3d;
mod server;

use ae3d::Window::Window;

use crate::ae3d::Mesh::Mesh;

fn main()
{
	Window::init("res/global/config.json");
	Window::resetDT();
	
	let mut m = Mesh::new();
	m.load("res/objects/test3d.obj".to_string());

	while Window::isOpen()
	{
		Window::update();
		Window::render();
	}
}
