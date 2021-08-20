#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod assets;
mod backend;
mod maze;
mod maze_gen;
mod rendering;
mod utils;

use miniquad::{KeyCode, conf::Conf};
use prelude::*;

mod prelude {
	pub use crate::{app::*, assets::*, backend::*, rendering::*, utils::*};
	pub use bevy_ecs_wasm::prelude::*;
	pub use crate::app::State;
	pub use miniquad::{warn, error, info};
}

pub fn main() {
	App::new()
		.insert_resource(Conf {
			window_width: 1024,
			window_height: 576,
			window_title: "Maze Walk".to_string(),
			sample_count: 2,
			..Default::default()
		})
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

#[cfg(target_arch = "wasm32")]
extern "C" {
	pub fn maze_assets_loaded();
}