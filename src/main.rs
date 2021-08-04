mod app;
mod assets;
mod backend;

use app::App;

pub fn main() {
	// App::new()
	// 	.with_plugin(miniquad::plugin)
	// 	.with_system(quit_on_esc)
	// 	.run();
	// App::build()

	// 	.add_plugins(DefaultPlugins)
	// 	.add_plugin(MiniquadPlugin)
	// 	.add_plugin(RenderingPlugin)
	// 	.add_plugin(maze::MazePlugin)
	// 	.add_system_to_stage(CoreStage::Last, quit_on_esc.system())
	// 	.run();
}

// fn quit_on_esc(world: &mut World) {
	
// 	for evt in input.iter() {
// 		if let Some(KeyCode::Escape) = evt.key_code {
// 			exit.send(AppExit {});
// 		}
// 	}
// }
	

// fn quit_on_esc(mut input: EventReader<KeyboardInput>, mut exit: EventWriter<AppExit>) {
// 	for evt in input.iter() {
// 		if let Some(KeyCode::Escape) = evt.key_code {
// 			exit.send(AppExit {});
// 		}
// 	}
// }
