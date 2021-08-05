use std::collections::HashSet;

use super::app::{Stage as UpdateStage, *};
use glam::{vec2, Vec2};
use legion::system;
use miniquad::{conf, date, Context, EventHandlerFree, KeyCode, UserData};

pub fn plugin(app: &mut AppBuilder) {
	app.set_runner(runner)
		.add_event::<WindowResized>()
		.add_event::<MouseMoved>()
		.add_event::<AppExit>()
		.add_system_to_stage(handle_exit_event_system(), UpdateStage::Last);
}

// resources
pub struct WindowSize(Vec2);

#[derive(Default)]
pub struct Time {
	startup_time: f64,
	last_update_time: Option<f64>,
	now: f64,
	delta: f64,
}
impl Time {
	pub fn seconds_since_startup(&self) -> f64 {
		self.now - self.startup_time
	}
	pub fn delta_seconds(&self) -> f64 {
		self.delta
	}
	fn update(s: &mut Stage) {
		let now = date::now();
		let mut t = s.app.get_resource::<Time>();
		t.now = now;
		t.delta = t
			.last_update_time
			.map(|last| (now - last).max(0.))
			.unwrap_or_default();
		t.last_update_time = Some(now);
	}
}

// events
pub struct WindowResized(Vec2);
pub struct MouseMoved(Vec2);
pub struct AppExit;

fn runner(mut app: App) {
	miniquad::start(conf::Conf::default(), |ctx| {
		app.resources.insert(WindowSize(ctx.screen_size().into()));
		app.resources.insert(ctx);
		app.resources.insert(Keyboard::default());
		app.resources.insert(Time {
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
		let size = vec2(width, height);
		self.app.get_resource::<WindowSize>().0 = size;
		self.app.emit_event(WindowResized(size));
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
		self.app.emit_event(MouseMoved(vec2(dx, dy)));
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

#[system]
fn handle_exit_event(#[resource] evt: &mut Event<AppExit>, #[resource] context: &Context) {
	if evt.iter().next().is_some() {
		context.request_quit();
	}
}
