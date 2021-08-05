use std::{iter, mem::take, slice::Iter};

use atomic_refcell::AtomicRefMut;
use legion::{
	system,
	systems::{Resource, Runnable, Step},
};
pub use legion::{Resources, Schedule, World};

#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Stage {
	First,
	AssetLoad,
	AssetEvents,
	PreUpdate,
	Update,
	PostUpdate,
	PreRender,
	Render,
	Last,
	EventReset,
}

type System = dyn Runnable + 'static;

pub struct App {
	pub world: World,
	pub resources: Resources,
	schedule: Schedule,
}

impl App {
	pub fn new() -> AppBuilder {
		AppBuilder {
			resources: Resources::default(),
			systems: vec![],
			startup_systems: vec![],
			runner: None,
		}
	}

	pub fn dispatch_update(&mut self) {
		self.schedule.execute(&mut self.world, &mut self.resources);
	}

	pub fn get_resource<T: 'static>(&mut self) -> AtomicRefMut<T> {
		self.resources.get_mut::<T>().unwrap()
	}

	pub fn get_event<T: 'static>(&mut self) -> AtomicRefMut<Event<T>> {
		self.resources.get_mut::<Event<T>>().unwrap()
	}

	pub fn emit_event<T: 'static>(&mut self, value: T) {
		self.get_event::<T>().emit(value)
	}
}

pub struct AppBuilder {
	resources: Resources,
	systems: Vec<(Box<System>, Stage)>,
	startup_systems: Vec<Box<System>>,
	runner: Option<Box<dyn FnOnce(App)>>,
}

impl AppBuilder {
	pub fn add_system(&mut self, s: impl Runnable + 'static) -> &mut Self {
		self.add_system_to_stage(s, Stage::Update)
	}

	pub fn add_startup_system(&mut self, s: impl Runnable + 'static) -> &mut Self {
		self.startup_systems.push(Box::new(s));
		self
	}

	pub fn add_system_to_stage(&mut self, s: impl Runnable + 'static, stage: Stage) -> &mut Self {
		self.systems.push((Box::new(s), stage));
		self
	}

	pub fn set_runner(&mut self, r: impl FnOnce(App) + 'static) -> &mut Self {
		self.runner = Some(Box::new(r));
		self
	}

	pub fn build(&mut self) -> App {
		let mut world = World::default();
		Into::<Schedule>::into(
			self.startup_systems
				.drain(..)
				.map(|s| Step::ThreadLocalSystem(s))
				.chain(iter::once(Step::FlushCmdBuffers))
				.collect::<Vec<_>>(),
		)
		.execute(&mut world, &mut self.resources);

		self.systems.sort_by_key(|(_, stage)| *stage);
		let steps: Vec<Step> = take(&mut self.systems)
			.into_iter()
			.map(|s| Step::ThreadLocalSystem(s.0))
			.collect();

		App {
			world,
			resources: take(&mut self.resources),
			schedule: steps.into(),
		}
	}

	pub fn run(&mut self) {
		let app = self.build();
		if let Some(runner) = self.runner.take() {
			(runner)(app);
		}
	}

	pub fn add_plugin(&mut self, p: impl Fn(&mut AppBuilder) + 'static) -> &mut Self {
		(p)(self);
		self
	}

	pub fn insert_resource(&mut self, r: impl Resource) -> &mut Self {
		self.resources.insert(r);
		self
	}

	pub fn add_event<T: 'static>(&mut self) -> &mut Self {
		self.resources.insert(Event::<T>::new());
		#[system]
		fn reset<T: 'static>(#[resource] e: &mut Event<T>) {
			e.clear()
		}
		self.add_system_to_stage(reset_system::<T>(), Stage::EventReset)
	}
}

pub struct Event<T> {
	values: Vec<T>,
}

impl<T> Event<T> {
	fn new() -> Self {
		Self {
			values: Vec::<_>::with_capacity(1),
		}
	}

	fn clear(&mut self) {
		self.values.clear();
	}

	pub fn emit(&mut self, value: T) {
		self.values.push(value);
	}

	pub fn iter(&self) -> Iter<T> {
		self.values.iter()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct Evt(i32);
	struct Count(i32);

	fn count(app: &App) -> i32 {
		app.resources.get::<Count>().unwrap().0
	}

	#[test]
	fn update_ticks() {
		#[system]
		fn increment(#[resource] c: &mut Count) {
			c.0 += 1;
		}

		let mut app = App::new()
			.insert_resource(Count(0))
			.add_system(increment_system())
			.build();

		assert_eq!(count(&app), 0);
		app.dispatch_update();
		assert_eq!(count(&app), 1);
		app.dispatch_update();
		assert_eq!(count(&app), 2);
	}

	#[test]
	fn event_cycle() {
		#[system]
		fn emit(#[resource] c: &Count, #[resource] evt: &mut Event<Evt>) {
			if c.0 == 0 {
				evt.emit(Evt(5));
			}
		}

		#[system]
		fn consume(#[resource] c: &mut Count, #[resource] evt: &mut Event<Evt>) {
			if let Some(val) = evt.iter().next().map(|e| e.0) {
				c.0 += val;
			}
		}

		let mut app = App::new()
			.insert_resource(Count(0))
			.add_event::<Evt>()
			.add_system(emit_system())
			.add_system(consume_system())
			.build();

		app.dispatch_update();
		assert_eq!(count(&app), 5, "first frame");
		app.dispatch_update();
		assert_eq!(count(&app), 5, "second frame");
	}

	#[test]
	fn update_stages() {
		#[derive(Default)]
		struct Calls(Vec<i32>);

		#[system]
		fn one(#[resource] c: &mut Calls) {
			c.0.push(1);
		}
		#[system]
		fn ten(#[resource] c: &mut Calls) {
			c.0.push(10);
		}
		#[system]
		fn hundred(#[resource] c: &mut Calls) {
			c.0.push(100);
		}

		let mut app = App::new()
			.insert_resource(Calls::default())
			.add_system_to_stage(one_system(), Stage::First)
			.add_system(hundred_system())
			.add_system_to_stage(ten_system(), Stage::PreUpdate)
			.build();
		app.dispatch_update();

		assert_eq!(&(app.resources.get::<Calls>().unwrap().0), &[1, 10, 100]);
	}
}
