#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod backend;
mod maze;
mod maze_gen;
mod rendering;
mod utils;

use miniquad::KeyCode;
use prelude::*;

mod prelude {
	pub use crate::{app::*, assets::*, backend::*, rendering::*, utils::*};
	pub use bevy_ecs::prelude::*;
}

pub fn main() {
	App::new()
		.add_plugin(backend::plugin)
		.add_plugin(rendering::plugin)
		.add_plugin(maze::plugin)
		.add_system(quit_on_esc.system())
		.run();
}

fn quit_on_esc(input: Res<Keyboard>, mut exit: EventWriter<AppExit>) {
	if input.was_just_pressed(KeyCode::Escape) {
		exit.send(AppExit {});
	}
}
