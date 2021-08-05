use bevy::utils::HashSet;
use glam::{Vec2, vec2};
use miniquad::{EventHandlerFree, UserData, conf, Context, KeyCode};
use super::app::*;

pub fn plugin(app: &mut AppBuilder) {
	app.set_runner(runner)
		.add_event::<WindowResized>()
		.add_event::<MouseMoved>()
		.add_event::<AppExit>();
}

// resources
pub struct WindowSize(Vec2);

// events
pub struct WindowResized(Vec2);
pub struct MouseMoved(Vec2);
pub struct AppExit;

fn runner(mut app: App) {
	miniquad::start(conf::Conf::default(), |ctx| {
        app.resources.insert(WindowSize(ctx.screen_size().into()));
		app.resources.insert(ctx);
		UserData::free(Stage{
			app
		})
	});
}

struct Stage {
	app: App
}

impl EventHandlerFree for Stage {
    fn update(&mut self) {
		if self.app.get_event::<AppExit>().iter().next().is_some() {
            self.app.get_resource::<Context>().request_quit();
        }
        self.app.dispatch_update();
		reset_just_pressed_keys(self);
    }

	fn resize_event(&mut self, width: f32, height: f32) {
		let size = vec2(width, height);
		self.app.get_resource::<WindowSize>().0 = size;
		self.app.emit_event(WindowResized(size));
	}

	fn key_down_event(&mut self, keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods, repeat: bool) {
		if !repeat {
			self.app.get_resource::<Keyboard>().toggle_key(keycode, true);
		}
	}

	fn key_up_event(&mut self, keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods) {
		self.app.get_resource::<Keyboard>().toggle_key(keycode, false);
	}

	fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
		self.app.emit_event(MouseMoved(vec2(dx, dy)));
	}

    fn draw(&mut self) {
    }
}

fn reset_just_pressed_keys(s: &mut Stage) {
	s.app.get_resource::<Keyboard>().just_pressed.clear();
}

#[derive(Default)]
pub struct Keyboard {
	just_pressed: HashSet<KeyCode>,
	held: HashSet<KeyCode>,
}

impl Keyboard {
	pub fn was_just_pressed(&self, k: KeyCode) -> bool {
		self.just_pressed.contains(&k)
	}

	pub fn is_pressed(&self, k: KeyCode) -> bool {
		self.held.contains(&k)
	}

	fn toggle_key(&mut self, k: KeyCode, pressed: bool) {
		if pressed {
			self.just_pressed.insert(k);
			self.held.insert(k);
		} else {
			self.held.remove(&k);
		}
	}
}