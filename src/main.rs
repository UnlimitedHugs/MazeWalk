mod cubes_demo;
mod quads_demo;
mod rendering;
mod utils;

use bevy::{
	app::{App, AppExit},
	input::keyboard::KeyboardInput,
	prelude::*,
};
use bevy_miniquad::MiniquadPlugin;
use rendering::RenderingPlugin;

pub fn main() {
	App::build()
		.add_plugins(DefaultPlugins)
		.add_plugin(MiniquadPlugin)
		.add_plugin(RenderingPlugin)
		//.add_plugin(quads_demo::QuadsDemoPlugin)
		.add_plugin(cubes_demo::CubesDemoPlugin)
		.add_system_to_stage(CoreStage::Last, quit_on_esc.system())
		.run();
}

fn quit_on_esc(mut input: EventReader<KeyboardInput>, mut exit: EventWriter<AppExit>) {
	for evt in input.iter() {
		if let Some(KeyCode::Escape) = evt.key_code {
			exit.send(AppExit {});
		}
	}
}
