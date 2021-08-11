use bevy_ecs::{component::Component, prelude::*};

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

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum AppState {
	Preload,
	Play,
}

impl Default for AppState {
	fn default() -> Self {
		AppState::Preload
	}
}

struct AppSystem {
	system: Box<dyn System<In = (), Out = ()>>,
	stage: CoreStage,
	state: Option<AppState>,
}

impl AppSystem {
	fn new(
		mut system: impl System<In = (), Out = ()>,
		mut world: &mut World,
		stage: CoreStage,
		state: Option<AppState>,
	) -> Self {
		Self::from_box(Box::new(system), world, stage, state)
	}

	fn from_box(
		mut system: Box<dyn System<In = (), Out = ()>>,
		mut world: &mut World,
		stage: CoreStage,
		state: Option<AppState>,
	) -> Self {
		system.initialize(&mut world);
		Self {
			system,
			stage,
			state,
		}
	}
}

#[derive(PartialEq)]
enum StateTransitionType {
	Enter,
	Exit,
}

struct StateListener {
	system: AppSystem,
	transition: StateTransitionType,
}

#[derive(Default)]
pub struct App {
	pub world: World,
	systems: Vec<AppSystem>,
	state_listeners: Vec<StateListener>,
}

impl App {
	pub fn new() -> AppBuilder {
		AppBuilder::new()
	}

	pub fn dispatch_update(&mut self) {
		let current_state = self.get_state().current;
		Self::run_systems(&mut self.systems, &mut self.world, Some(current_state));
		self.world.check_change_ticks();
		self.world.clear_trackers();
		self.apply_state_transition();
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

	fn run_systems(systems: &mut [AppSystem], world: &mut World, current_state: Option<AppState>) {
		let mut pending_buffers = vec![];
		for sys in systems.iter_mut() {
			if sys.state.is_some() && sys.state != current_state {
				continue;
			}
			sys.system.run((), world);
			pending_buffers.push(sys);
		}
		for sys in pending_buffers.into_iter() {
			sys.system.apply_buffers(world);
		}
	}

	fn get_state(&mut self) -> Mut<State> {
		self.world.get_resource_mut::<State>().unwrap()
	}

	fn apply_state_transition(&mut self) {
		let State { current, pending } = *self.get_state();
		if let Some(next) = pending {
			self.call_state_listeners(current, StateTransitionType::Exit);
			self.get_state().current = next;
			self.call_state_listeners(next, StateTransitionType::Enter);
		}
	}

	fn call_state_listeners(&mut self, state: AppState, transition: StateTransitionType) {
		for listener in self.state_listeners.iter_mut() {
			let StateListener {
				system,
				transition: system_transition,
			} = listener;
			if system.state == Some(state) && *system_transition == transition {
				system.system.run((), &mut self.world);
				system.system.apply_buffers(&mut self.world);
			}
		}
	}
}

pub struct AppBuilder {
	pub world: Option<World>,
	startup_systems: Vec<AppSystem>,
	systems: Vec<AppSystem>,
	runner: Option<Box<dyn FnOnce(App)>>,
	state_listeners: Vec<StateListener>,
}

impl AppBuilder {
	fn new() -> Self {
		Self {
			world: Some(Default::default()),
			startup_systems: Default::default(),
			systems: Default::default(),
			runner: None,
			state_listeners: Default::default(),
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
		let s = AppSystem::new(system, self.world(), stage, None);
		self.systems.push(s);
		self
	}

	pub fn add_startup_system(&mut self, system: impl System<In = (), Out = ()>) -> &mut Self {
		let s = AppSystem::new(system, self.world(), CoreStage::First, None);
		self.startup_systems.push(s);
		self
	}

	pub fn add_system_stateful(
		&mut self,
		state: AppState,
		system: impl System<In = (), Out = ()>,
	) -> &mut Self {
		let s = AppSystem::new(system, self.world(), CoreStage::Update, Some(state));
		self.systems.push(s);
		self
	}

	pub fn add_system_list(
		&mut self,
		stage: CoreStage,
		state: Option<AppState>,
		list: SystemList,
	) -> &mut Self {
		for sys in list.systems.into_iter() {
			let s = AppSystem::from_box(sys, self.world(), stage, state);
			self.systems.push(s);
		}
		self
	}

	pub fn on_enter_state(
		&mut self,
		state: AppState,
		system: impl System<In = (), Out = ()>,
	) -> &mut Self {
		self.add_state_listener(state, StateTransitionType::Enter, system);
		self
	}

	pub fn on_exit_state(
		&mut self,
		state: AppState,
		system: impl System<In = (), Out = ()>,
	) -> &mut Self {
		self.add_state_listener(state, StateTransitionType::Exit, system);
		self
	}

	fn add_state_listener(
		&mut self,
		state: AppState,
		transition: StateTransitionType,
		system: impl System<In = (), Out = ()>,
	) {
		let listener = StateListener {
			system: AppSystem::new(system, self.world(), CoreStage::Last, Some(state)),
			transition,
		};
		self.state_listeners.push(listener);
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
		self.world().insert_resource(resource);
		self
	}

	pub fn init_resource<R>(&mut self) -> &mut Self
	where
		R: FromWorld + Send + Sync + 'static,
	{
		let world = self.world();
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
		let mut world = self.world.take().unwrap();
		let state = State::default();
		let current_state = state.current;
		world.insert_resource(state);
		App::run_systems(
			&mut self.startup_systems.drain(..).collect::<Vec<_>>(),
			&mut world,
			None,
		);

		let systems: Vec<_> = {
			self.systems.sort_by_key(|sys| sys.stage);
			self.systems.drain(..).collect::<Vec<_>>()
		};

		let state_listeners = self.state_listeners.drain(..).collect::<Vec<_>>();

		let mut app = App {
			world,
			systems,
			state_listeners,
		};

		app.call_state_listeners(current_state, StateTransitionType::Enter);
		app
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

	pub fn world(&mut self) -> &mut World {
		self.world.as_mut().unwrap()
	}
}

#[derive(Default)]
struct State {
	current: AppState,
	pending: Option<AppState>,
}

impl State {
	pub fn get_current(&self) -> AppState {
		self.current
	}

	pub fn schedule_transition(&mut self, new_state: AppState) {
		self.pending = Some(new_state);
	}
}

pub struct SystemList {
	systems: Vec<Box<dyn System<In = (), Out = ()>>>,
}

impl SystemList {
	pub fn new() -> Self {
		Self {
			systems: Default::default(),
		}
	}

	pub fn with(mut self, system: impl System<In = (), Out = ()>) -> Self {
		self.systems.push(Box::new(system));
		self
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_ecs::system::BoxedSystem;

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
			.add_system_list(
				CoreStage::Update,
				None,
				SystemList::new().with(emit.system()).with(consume.system()),
			)
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

	#[test]
	fn state_transition() {
		use super::State;
		use {AppState::*, CallType::*};

		#[derive(Default)]
		struct Calls(Vec<CallType>);
		#[derive(Clone, Copy, PartialEq, Debug)]
		enum CallType {
			Startup,
			UpdateState(AppState),
			Update,
			Enter(AppState),
			Exit(AppState),
		}

		fn stateful(s: Res<State>, mut c: ResMut<Calls>) {
			c.0.push(UpdateState(s.current));
		}
		fn enter(s: Res<State>, mut c: ResMut<Calls>) {
			c.0.push(Enter(s.current));
		}
		fn exit(s: Res<State>, mut c: ResMut<Calls>) {
			c.0.push(Exit(s.current));
		}
		fn take_calls(app: &mut App) -> Vec<CallType> {
			app.world
				.get_resource_mut::<Calls>()
				.unwrap()
				.0
				.drain(..)
				.collect::<Vec<_>>()
		}
		fn schedule_transition(app: &mut App, s: AppState) {
			app.world
				.get_resource_mut::<State>()
				.unwrap()
				.schedule_transition(s);
		}

		let mut app = App::new()
			.insert_resource(Calls::default())
			.add_startup_system((|mut c: ResMut<Calls>| c.0.push(Startup)).system())
			.add_system_stateful(Play, stateful.system())
			.add_system_stateful(Preload, stateful.system())
			.add_system((|mut c: ResMut<Calls>| c.0.push(Update)).system())
			.on_enter_state(Preload, enter.system())
			.on_exit_state(Preload, exit.system())
			.on_enter_state(Play, enter.system())
			.on_exit_state(Play, exit.system())
			.build();

		assert_eq!(take_calls(&mut app), &[Startup, Enter(Preload)]);

		app.dispatch_update();
		assert_eq!(take_calls(&mut app), &[UpdateState(Preload), Update]);

		schedule_transition(&mut app, Play);
		app.dispatch_update();
		assert_eq!(
			take_calls(&mut app),
			&[UpdateState(Preload), Update, Exit(Preload), Enter(Play)]
		);

		schedule_transition(&mut app, Play);
		app.dispatch_update();
		assert_eq!(
			take_calls(&mut app),
			&[UpdateState(Play), Update, Exit(Play), Enter(Play)]
		);
	}
}
