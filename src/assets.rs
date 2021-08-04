use std::{
	collections::HashMap,
	marker::PhantomData,
	mem::{swap, take},
	sync::Arc,
};

use super::app::*;
use bevy_hecs::Component;

pub fn plugin(app: &mut App) {}

impl App {
	fn add_asset_type<T: Component>(self) -> Self {
		self.insert_resource(Assets::<T>::new())
			.add_event::<AssetEvent<T>>()
			.add_system_to_stage(update_assets::<T>, Stage::AssetLoad)
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

fn update_assets<T: Component>(w: &mut World) {
	let dropped = {
		let mut assets = w.get_resource_mut::<Assets<T>>();
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
		let mut evt = w.get_event::<AssetEvent<T>>();
		for handle in handles.into_iter() {
			evt.emit(AssetEvent::Removed(handle));
		}
	}
	let created: Option<Vec<Handle<T>>> = {
		let mut assets = w.get_resource_mut::<Assets<T>>();
		if assets.pending_created_events.len() > 0 {
			Some(take(&mut assets.pending_created_events))
		} else {
			None
		}
	};
	if let Some(handles) = created {
		let mut evt = w.get_event::<AssetEvent<T>>();
		for handle in handles.into_iter() {
			evt.emit(AssetEvent::Added(handle));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use bevy_hecs::Mut;

	#[derive(Default)]
	struct IntEvents(Vec<i32>);

	fn read(a: &mut App) -> &[i32] {
		&a.world.get_resource::<IntEvents>().0
	}

	fn assets(w: &mut World) -> Mut<Assets<i32>> {
		w.get_resource_mut::<Assets<i32>>()
	}

	#[test]
	fn asset_lifecycle() {
		use super::AssetEvent::*;
		let log_events = |w: &mut World| {
			let nums = {
				w.get_resource::<Event<AssetEvent<i32>>>()
					.iter()
					.map(|e| {
						w.get_resource::<Assets<i32>>()
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
			w.get_resource_mut::<IntEvents>().0.extend(nums);
		};

		let mut app = App::new()
			.add_asset_type::<i32>()
			.insert_resource(IntEvents::default())
			.add_system_to_stage(log_events, Stage::AssetEvents)
			.run();
		let _one = assets(&mut app.world).create(1);
		{
			let _two = assets(&mut app.world).create(2);
			let _three = assets(&mut app.world).create(3);
			app.dispatch_update();
			assert_eq!(read(&mut app), &[1, 2, 3], "frame 1");
		}
		app.dispatch_update();
		assert_eq!(read(&mut app), &[1, 2, 3, -2, -3], "frame 2");
	}
}
