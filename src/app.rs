use std::slice::Iter;

use bevy_hecs::{Component, Mut};

pub use bevy_hecs::World;

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
}

pub struct App {
	pub world: World,
	systems: Vec<Box<dyn FnMut(&mut World)>>,
	runner: Option<Box<dyn FnOnce(App)>>,
	pending_systems: Vec<(Box<dyn FnMut(&mut World)>, Stage)>,
}

impl App {
	pub fn new() -> Self {
		Self {
			world: World::new(),
			systems: vec![],
			runner: None,
			pending_systems: vec![],
		}
	}

	pub fn add_system(self, s: impl FnMut(&mut World) + 'static) -> Self {
		self.add_system_to_stage(s, Stage::Update)
	}

	pub fn add_system_to_stage(
		mut self,
		s: impl FnMut(&mut World) + 'static,
		stage: Stage,
	) -> Self {
		self.pending_systems.push((Box::new(s), stage));
		self
	}

	pub fn dispatch_update(&mut self) {
		for c in self.systems.iter_mut() {
			(c)(&mut self.world);
		}
	}

	pub fn set_runner(mut self, r: impl FnOnce(App) + 'static) -> Self {
		self.runner = Some(Box::new(r));
		self
	}

	pub fn run(mut self) -> Self {
		self.pending_systems.sort_by_key(|(_, stage)| *stage);
		self.systems
			.extend(self.pending_systems.into_iter().map(|(s, _)| s));
		self.pending_systems = Vec::with_capacity(0);

		if let Some(runner) = self.runner.take() {
			(runner)(self);
			unreachable!();
		} else {
			return self;
		}
	}

	pub fn add_plugin(&mut self, p: impl Fn(&mut App) + 'static) -> &mut Self {
		(p)(self);
		self
	}

	pub fn insert_resource(mut self, r: impl Component) -> Self {
		self.world.insert_resource(r);
		self
	}

	pub fn add_event<T: Send + Sync + 'static>(mut self) -> Self {
		self.world.insert_resource(Event::<T>::new());
		self.add_system_to_stage(|w: &mut World| w.get_event::<T>().clear(), Stage::Last)
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

pub trait WorldExtensions {
	fn insert_resource<T: Component>(&mut self, r: T) -> &mut Self;
	fn get_resource<T: Component>(&self) -> &T;
	fn get_resource_mut<T: Component>(&mut self) -> Mut<T>;
	fn get_event<T: Component>(&mut self) -> Mut<Event<T>>;
}

impl WorldExtensions for World {
	fn insert_resource<T: Component>(&mut self, res: T) -> &mut Self {
		if let Some(mut r) = self.query_mut::<&mut T>().next() {
			*r = res;
		} else {
			self.spawn((res,));
		}
		debug_assert_eq!(
			self.query_mut::<&mut T>().count(),
			1,
			"Duplicate resource {}",
			std::any::type_name::<T>()
		);
		self
	}

	fn get_resource<T: Component>(&self) -> &T {
		for r in self.query::<&T>() {
			return r;
		}
		panic_on_resource::<T>();
		unreachable!();
	}

	fn get_resource_mut<T: Component>(&mut self) -> Mut<T> {
		for r in self.query_mut::<&mut T>() {
			return r;
		}
		panic_on_resource::<T>();
		unreachable!();
	}

	fn get_event<T: Component>(&mut self) -> Mut<Event<T>> {
		self.get_resource_mut::<Event<T>>()
	}
}

fn panic_on_resource<T>() {
	#[cfg(debug_assertions)]
	panic!("Resource not found: {}", std::any::type_name::<T>());
	#[cfg(not(debug_assertions))]
	panic!("Resource not found");
}

#[cfg(test)]
mod tests {
	use super::*;

	struct Evt(i32);
	struct Count(i32);

	fn count(app: &App) -> i32 {
		(*app.world.get_resource::<Count>()).0
	}

	fn increment(w: &mut World, inc: i32) {
		(*w.get_resource_mut::<Count>()).0 += inc;
	}

	#[test]
	fn update_ticks() {
		let update = |w: &mut World| {
			increment(w, 1);
		};

		let mut app = App::new()
			.insert_resource(Count(0))
			.add_system(update)
			.run();

		assert_eq!(count(&app), 0);
		app.dispatch_update();
		assert_eq!(count(&app), 1);
		app.dispatch_update();
		assert_eq!(count(&app), 2);
	}

	#[test]
	fn event_cycle() {
		let emit = |w: &mut World| {
			if w.get_resource::<Count>().0 == 0 {
				w.get_event::<Evt>().emit(Evt(5));
			}
		};

		let consume = |w: &mut World| {
			let evt_value = w.get_event::<Evt>().iter().next().map(|e| e.0);
			if let Some(val) = evt_value {
				increment(w, val);
			}
		};

		let mut app = App::new()
			.insert_resource(Count(0))
			.add_event::<Evt>()
			.add_system(emit)
			.add_system(consume)
			.run();

		app.dispatch_update();
		assert_eq!(count(&app), 5, "first frame");
		app.dispatch_update();
		assert_eq!(count(&app), 5, "second frame");
	}

	#[test]
	fn update_stages() {
		#[derive(Default)]
		struct Calls(Vec<i32>);
		fn add(w: &mut World, i: i32) {
			w.get_resource_mut::<Calls>().0.push(i);
		}

		let one = |w: &mut World| add(w, 1);
		let ten = |w: &mut World| add(w, 10);
		let hundred = |w: &mut World| add(w, 100);

		let mut app = App::new()
			.insert_resource(Calls::default())
			.add_system_to_stage(one, Stage::First)
			.add_system(hundred)
			.add_system_to_stage(ten, Stage::PreUpdate)
			.run();
		app.dispatch_update();

		assert_eq!(&(app.world.get_resource::<Calls>().0), &[1, 10, 100]);
	}
}
