use std::collections::HashSet;

use super::app::*;
use bevy_ecs::prelude::*;
use miniquad::{Context, EventHandlerFree, KeyCode, UserData, conf::Conf, date};

pub fn plugin(app: &mut AppBuilder) {
	app.set_runner(runner)
		.add_event::<WindowResize>()
		.add_event::<MouseMove>()
		.add_event::<MouseScroll>()
		.add_event::<AppExit>()
		.add_system_to_stage(CoreStage::Last, handle_exit_event.system());
}

// resources
pub struct WindowSize {
	pub width: f32,
	pub height: f32,
}
impl WindowSize {
	fn new((width, height): (f32, f32)) -> Self {
		Self { width, height }
	}
}

#[derive(Default)]
pub struct Time {
	startup_time: f64,
	last_update_time: Option<f64>,
	now: f64,
	delta: f32,
}
impl Time {
	pub fn seconds_since_startup(&self) -> f64 {
		self.now - self.startup_time
	}
	pub fn delta_seconds(&self) -> f32 {
		self.delta
	}
	fn update(s: &mut Stage) {
		let now = date::now();
		let mut t = s.app.get_resource::<Time>();
		t.now = now;
		t.delta = t
			.last_update_time
			.map(|last| (now - last).max(0.) as f32)
			.unwrap_or_default();
		t.last_update_time = Some(now);
	}
}

// events
pub struct WindowResize {
	pub width: f32,
	pub height: f32,
}
pub struct MouseMove {
	pub dx: f32,
	pub dy: f32,
}
pub struct MouseScroll {
	pub delta: f32,
}
pub struct AppExit;

fn runner(mut app: App) {
	let conf = app.world.remove_resource::<Conf>().unwrap_or_default();
	miniquad::start(conf, |ctx| {
		app.world
			.insert_resource(WindowSize::new(ctx.screen_size()));
		app.world.insert_resource(ctx);
		app.world.insert_resource(Keyboard::default());
		app.world.insert_resource(Time {
			startup_time: date::now(),
			..Default::default()
		});
		UserData::free(Stage { app })
	});
}

struct Stage {
	app: App,
}

impl EventHandlerFree for Stage {
	fn update(&mut self) {
		Time::update(self);
		self.app.dispatch_update();
		Keyboard::update(self);
	}

	fn resize_event(&mut self, width: f32, height: f32) {
		*self.app.get_resource::<WindowSize>() = WindowSize { width, height };
		self.app.emit_event(WindowResize { width, height });
	}

	fn key_down_event(
		&mut self,
		keycode: miniquad::KeyCode,
		_keymods: miniquad::KeyMods,
		repeat: bool,
	) {
		if !repeat {
			self.app
				.get_resource::<Keyboard>()
				.toggle_key(keycode, true);
		}
	}

	fn key_up_event(&mut self, keycode: miniquad::KeyCode, _keymods: miniquad::KeyMods) {
		self.app
			.get_resource::<Keyboard>()
			.toggle_key(keycode, false);
	}

	fn raw_mouse_motion(&mut self, dx: f32, dy: f32) {
		self.app.emit_event(MouseMove { dx, dy });
	}

	fn mouse_wheel_event(&mut self, _x: f32, delta: f32) {
		self.app.emit_event(MouseScroll { delta });
	}

	fn draw(&mut self) {}
}

#[derive(Default, Debug)]
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

	pub fn get_just_pressed(&self) -> impl ExactSizeIterator<Item = &KeyCode> {
		self.just_pressed.iter()
	}

	fn toggle_key(&mut self, k: KeyCode, pressed: bool) {
		if pressed {
			self.just_pressed.insert(k);
			self.held.insert(k);
		} else {
			self.held.remove(&k);
		}
	}

	fn update(s: &mut Stage) {
		s.app.get_resource::<Keyboard>().just_pressed.clear();
	}
}

fn handle_exit_event(mut evt: EventReader<AppExit>, context: Res<Context>) {
	if evt.iter().next().is_some() {
		context.request_quit();
	}
}
