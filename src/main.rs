mod app;
mod assets;
mod backend;
mod rendering;
mod utils;

use miniquad::KeyCode;
use prelude::*;

mod prelude {
	pub use crate::{app::*, assets::*, backend::*, rendering::*};
	pub use bevy_ecs::prelude::*;
}

pub fn main() {
	App::new()
		.add_plugin(backend::plugin)
		.add_plugin(rendering::plugin)
		.add_system(quit_on_esc.system())
		.run();
}

fn quit_on_esc(input: Res<Keyboard>, mut exit: EventWriter<AppExit>) {
	if input.was_just_pressed(KeyCode::Escape) {
		exit.send(AppExit {});
	}
}
