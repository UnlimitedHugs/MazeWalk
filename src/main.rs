mod app;
mod assets;
mod backend;
mod quads_demo;
mod rendering;
mod utils;

use app::*;
use backend::{AppExit, Keyboard};
use legion::system;
use miniquad::KeyCode;

pub fn main() {
	App::new()
		.add_plugin(backend::plugin)
		.add_plugin(rendering::plugin)
		.add_plugin(quads_demo::plugin)
		.add_system(quit_on_esc_system())
		.run();
}

#[system]
fn quit_on_esc(#[resource] key: &Keyboard, #[resource] exit: &mut Event<AppExit>) {
	if key.was_just_pressed(KeyCode::Escape) {
		exit.emit(AppExit);
	}
}
