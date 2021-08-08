use bevy_ecs::{component::Component, prelude::*};

#[allow(dead_code)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum CoreStage {
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

type SysFn = dyn System<In = (), Out = ()>;

#[derive(Default)]
pub struct App {
	pub world: World,
	systems: Vec<Box<SysFn>>,
}

impl App {
	pub fn new() -> AppBuilder {
		AppBuilder::new()
	}

	pub fn dispatch_update(&mut self) {
		Self::run_systems(&mut self.systems, &mut self.world);
		self.world.check_change_ticks();
		self.world.clear_trackers();
	}

	pub fn get_resource<T: Component>(&mut self) -> Mut<T> {
		self.world.get_resource_mut::<T>().unwrap()
	}

	pub fn get_event<T: Component>(&mut self) -> Mut<Events<T>> {
		self.get_resource::<Events<T>>()
	}

	pub fn emit_event<T: Component>(&mut self, value: T) {
		self.get_event::<T>().send(value)
	}

	fn run_systems(systems: &mut [Box<SysFn>], world: &mut World) {
		for sys in systems.iter_mut() {
			sys.run((), world);
			sys.apply_buffers(world);
		}
	}
}

pub struct AppBuilder {
	world: Option<World>,
	startup_systems: Vec<Box<SysFn>>,
	systems: Vec<(Box<SysFn>, CoreStage)>,
	runner: Option<Box<dyn FnOnce(App)>>,
}

impl AppBuilder {
	fn new() -> Self {
		Self {
			world: Some(Default::default()),
			startup_systems: Default::default(),
			systems: Default::default(),
			runner: None,
		}
	}

	pub fn add_system(&mut self, system: impl System<In = (), Out = ()>) -> &mut Self {
		self.add_system_to_stage(CoreStage::Update, system)
	}

	pub fn add_system_to_stage(
		&mut self,
		stage: CoreStage,
		system: impl System<In = (), Out = ()>,
	) -> &mut Self {
		self.systems.push((Box::new(system), stage));
		self
	}

	pub fn add_startup_system(&mut self, system: impl System<In = (), Out = ()>) -> &mut Self {
		self.startup_systems.push(Box::new(system));
		self
	}

	pub fn add_event<T>(&mut self) -> &mut Self
	where
		T: Component,
	{
		self.insert_resource(Events::<T>::default())
			.add_system_to_stage(CoreStage::First, Events::<T>::update_system.system())
	}

	pub fn insert_resource<T>(&mut self, resource: T) -> &mut Self
	where
		T: Component,
	{
		self.world.as_mut().unwrap().insert_resource(resource);
		self
	}

	pub fn init_resource<R>(&mut self) -> &mut Self
	where
		R: FromWorld + Send + Sync + 'static,
	{
		let world = self.world.as_mut().unwrap();
		if !world.contains_resource::<R>() {
			let resource = R::from_world(world);
			self.insert_resource(resource);
		}
		self
	}

	pub fn set_runner(&mut self, r: impl FnOnce(App) + 'static) -> &mut Self {
		self.runner = Some(Box::new(r));
		self
	}

	pub fn build(&mut self) -> App {
		let init_system = |mut sys: Box<SysFn>, w: &mut World| {
			sys.initialize(w);
			sys
		};

		let mut world = self.world.take().unwrap();
		App::run_systems(
			&mut self
				.startup_systems
				.drain(..)
				.map(|s| init_system(s, &mut world))
				.collect::<Vec<_>>(),
			&mut world,
		);
		let systems: Vec<_> = {
			let mut s = self.systems.drain(..).collect::<Vec<_>>();
			s.sort_by_key(|(_, stage)| *stage);
			s.into_iter()
				.map(|(sys, _)| sys)
				.map(|s| init_system(s, &mut world))
				.collect()
		};

		App { world, systems }
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
}

#[cfg(test)]
mod tests {
	use super::*;

	struct Evt(i32);
	#[derive(Debug)]
	struct Count(i32);

	fn count(app: &App) -> i32 {
		app.world.get_resource::<Count>().unwrap().0
	}

	#[test]
	fn update_ticks() {
		fn increment(mut c: ResMut<Count>) {
			c.0 += 1;
		}

		let mut app = App::new()
			.insert_resource(Count(0))
			.add_system(increment.system())
			.build();

		assert_eq!(count(&app), 0);
		app.dispatch_update();
		assert_eq!(count(&app), 1);
		app.dispatch_update();
		assert_eq!(count(&app), 2);
	}

	#[test]
	fn event_cycle() {
		fn emit(c: Res<Count>, mut evt: EventWriter<Evt>) {
			if c.0 == 0 {
				evt.send(Evt(5));
			}
		}

		fn consume(mut c: ResMut<Count>, mut evt: EventReader<Evt>) {
			if let Some(val) = evt.iter().next().map(|e| e.0) {
				c.0 += val;
			}
		}

		let mut app = App::new()
			.insert_resource(Count(0))
			.add_event::<Evt>()
			.add_system(emit.system())
			.add_system(consume.system())
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

		fn one(mut c: ResMut<Calls>) {
			c.0.push(1);
		}
		fn ten(mut c: ResMut<Calls>) {
			c.0.push(10);
		}
		fn hundred(mut c: ResMut<Calls>) {
			c.0.push(100);
		}

		let mut app = App::new()
			.insert_resource(Calls::default())
			.add_system_to_stage(CoreStage::First, one.system())
			.add_system(hundred.system())
			.add_system_to_stage(CoreStage::PreUpdate, ten.system())
			.build();
		app.dispatch_update();

		assert_eq!(
			&(app.world.get_resource::<Calls>().unwrap().0),
			&[1, 10, 100]
		);
	}
}
