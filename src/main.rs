#![allow(non_snake_case, static_mut_refs, dead_code)]
// #![windows_subsystem = "windows"]
mod ae3d;
mod envell;

use ae3d::Window::Window;

fn main()
{
	Window::init("res/global/game.json");
	Window::resetDT();
	while Window::isOpen()
	{
		Window::update();
		Window::render();
	}
}
