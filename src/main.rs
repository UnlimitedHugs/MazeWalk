mod rendering;
mod quads_demo;
mod shape;

use bevy::{app::App, prelude::*};
use bevy_miniquad::MiniquadPlugin;
use rendering::RenderingPlugin;

pub fn main() {
	App::build()
		.add_plugins(DefaultPlugins)
		.add_plugin(MiniquadPlugin)
		.add_plugin(RenderingPlugin)
		.add_plugin(quads_demo::QuadsDemoPlugin)
		.run();
}
