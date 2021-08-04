use std::{collections::HashMap, marker::PhantomData, mem::swap, sync::Arc};

use super::app::*;
use legion::system;

pub fn plugin(app: &mut AppBuilder) {}

impl AppBuilder {
	fn add_asset_type<T: 'static>(self) -> Self {
		self.insert_resource(Assets::<T>::new())
			.add_event::<AssetEvent<T>>()
			.add_system_to_stage(update_assets_system::<T>(), Stage::AssetLoad)
	}
}

#[derive(Debug, PartialEq)]
pub enum AssetEvent<T> {
	Added(Handle<T>),
	Removed(Handle<T>),
}

#[derive(Debug)]
pub struct Handle<T> {
	id: Arc<u32>,
	_p: PhantomData<T>,
}

impl<T> Handle<T> {
	fn new(id: u32) -> Self {
		Self {
			id: Arc::new(id),
			_p: PhantomData,
		}
	}
	pub fn id(&self) -> u32 {
		*self.id
	}
}
impl<T> Clone for Handle<T> {
	fn clone(&self) -> Self {
		Handle {
			id: self.id.clone(),
			_p: PhantomData,
		}
	}
}
impl<T> PartialEq for Handle<T> {
	fn eq(&self, other: &Self) -> bool {
		self.id == other.id
	}
}

struct Assets<T> {
	handles: Vec<Handle<T>>,
	values: HashMap<u32, T>,
	last_id: u32,
	pending_created_events: Vec<Handle<T>>,
}

impl<T> Assets<T> {
	fn new() -> Self {
		Assets {
			handles: vec![],
			values: HashMap::new(),
			last_id: 0,
			pending_created_events: vec![],
		}
	}

	pub fn create(&mut self, asset: T) -> Handle<T> {
		self.last_id += 1;
		let id = self.last_id;
		let handle = Handle::new(id);
		self.handles.push(handle.clone());
		self.values.insert(id, asset);
		self.pending_created_events.push(handle.clone());
		handle
	}

	pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
		self.values.get(&handle.id())
	}
}

#[system]
fn update_assets<T: 'static>(
	#[resource] assets: &mut Assets<T>,
	#[resource] evt: &mut Event<AssetEvent<T>>,
) {
	for handle in assets.pending_created_events.drain(..) {
		evt.emit(AssetEvent::Added(handle));
	}
	let dropped = {
		let mut dropped = Option::<Vec<Handle<T>>>::None;
		let mut kept_handles = vec![];
		for handle in assets.handles.drain(..) {
			if Arc::strong_count(&handle.id) <= 1 {
				dropped = Some({
					let mut v = dropped.unwrap_or_else(|| vec![]);
					v.push(handle);
					v
				});
			} else {
				kept_handles.push(handle);
			}
		}
		swap(&mut assets.handles, &mut kept_handles);
		dropped
	};
	if let Some(handles) = dropped {
		for handle in handles.into_iter() {
			evt.emit(AssetEvent::Removed(handle));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use atomic_refcell::AtomicRefMut;

	#[derive(Default)]
	struct IntEvents(Vec<i32>);

	fn read(a: &mut App) -> Vec<i32> {
		a.resources
			.get::<IntEvents>()
			.unwrap()
			.0
			.iter()
			.copied()
			.collect::<Vec<_>>()
	}

	fn assets(a: &mut App) -> AtomicRefMut<Assets<i32>> {
		a.resources.get_mut::<Assets<i32>>().unwrap()
	}

	#[test]
	fn asset_lifecycle() {
		use super::AssetEvent::*;
		#[system]
		fn log_events(
			#[resource] evt: &Event<AssetEvent<i32>>,
			#[resource] assets: &mut Assets<i32>,
			#[resource] events: &mut IntEvents,
		) {
			let nums = {
				evt.iter()
					.map(|e| {
						assets
							.get(match e {
								Added(h) | Removed(h) => &h,
							})
							.unwrap() * match e {
							Added(_) => 1,
							Removed(_) => -1,
						}
					})
					.collect::<Vec<_>>()
			};
			events.0.extend(nums);
		}

		let app = &mut App::new()
			.add_asset_type::<i32>()
			.insert_resource(IntEvents::default())
			.add_system_to_stage(log_events_system(), Stage::AssetEvents)
			.run();
		let _one = assets(app).create(1);
		{
			let _two = assets(app).create(2);
			let _three = assets(app).create(3);
			app.dispatch_update();
			assert_eq!(read(app), &[1, 2, 3], "frame 1");
		}
		app.dispatch_update();
		assert_eq!(read(app), &[1, 2, 3, -2, -3], "frame 2");
	}
}
