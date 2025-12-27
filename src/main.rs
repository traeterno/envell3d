#![allow(non_snake_case, static_mut_refs, dead_code)]
// #![windows_subsystem = "windows"]
mod ae3d;
mod server;

use ae3d::Window::Window;

fn main()
{
	Window::init("res/global/config.json");
	Window::resetDT();

	let cam = Window::getCamera();
	cam.toggleTransform(true);
	while Window::isOpen()
	{

		Window::update();
		Window::render();
	}
}
