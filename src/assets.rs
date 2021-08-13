use bevy_ecs::component::Component;
use std::{
	any::type_name,
	collections::HashMap,
	marker::PhantomData,
	mem::swap,
	sync::{Arc, Mutex},
};

use crate::prelude::*;

impl AppBuilder {
	pub fn add_asset_type<T: Component>(&mut self) -> &mut Self {
		self.add_asset_type_with_loader::<T, _>(MiniquadFileLoader {})
	}

	fn add_asset_type_with_loader<T: Component, FL: FileLoader>(
		&mut self,
		loader: FL,
	) -> &mut Self {
		self.insert_resource(Assets::<T>::new(loader))
			.add_event::<AssetEvent<T>>()
			.add_system_to_stage(CoreStage::AssetLoad, update_assets::<T>.system())
	}

	pub fn use_asset_processor<T: Component>(
		&mut self,
		loader: impl Fn(Vec<u8>) -> Result<T, String> + 'static + Send + Sync,
	) -> &mut Self {
		self.world
			.as_mut()
			.unwrap()
			.get_resource_mut::<Assets<T>>()
			.unwrap()
			.use_processor(loader);
		self
	}
}

#[derive(Debug, PartialEq)]
pub enum AssetEvent<T> {
	Added(Handle<T>),
	Removed(Handle<T>),
}

pub type HandleId = u32;

#[derive(Debug)]
pub struct Handle<T> {
	id: Arc<HandleId>,
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

pub struct Assets<T: Component> {
	handles: Vec<Handle<T>>,
	values: HashMap<HandleId, T>,
	last_id: HandleId,
	pending_created_events: Vec<Handle<T>>,
	pending_loaded_files: Arc<Mutex<Vec<LoadedFile<T>>>>,
	processor: Option<Processor<T>>,
	loader: Box<dyn FileLoader>,
}

pub type Processor<T> = Box<dyn Fn(Vec<u8>) -> Result<T, String> + Send + Sync>;

struct LoadedFile<T> {
	handle: Handle<T>,
	path: String,
	bytes: Vec<u8>,
}

impl<T: Component> Assets<T> {
	fn new(loader: impl FileLoader) -> Self {
		Self {
			handles: vec![],
			values: HashMap::new(),
			last_id: 0,
			pending_created_events: vec![],
			pending_loaded_files: Default::default(),
			processor: None,
			loader: Box::new(loader),
		}
	}

	pub fn add(&mut self, value: T) -> Handle<T> {
		let handle = self.create_handle();
		self.insert_asset(&handle, value);
		handle
	}

	pub fn get(&self, handle: &Handle<T>) -> Option<&T> {
		self.values.get(&handle.id())
	}

	pub fn load(&mut self, path: &str) -> Handle<T> {
		let handle = self.create_handle();
		let handle_clone = handle.clone();
		let path_string = path.to_string();
		let files = Arc::clone(&self.pending_loaded_files);
		self.loader.load(
			path,
			Box::new(move |result| match result {
				Ok(bytes) => files.lock().unwrap().push(LoadedFile {
					handle: handle_clone.clone(),
					path: path_string.clone(),
					bytes,
				}),
				Err(e) => error!("Failed to load {}: {}", path_string, e),
			}),
		);
		handle
	}

	fn create_handle(&mut self) -> Handle<T> {
		self.last_id += 1;
		let id = self.last_id;
		let handle = Handle::new(id);
		self.handles.push(handle.clone());
		handle
	}

	fn insert_asset(&mut self, handle: &Handle<T>, value: T) {
		self.values.insert(handle.id(), value);
		self.pending_created_events.push(handle.clone());
	}

	fn use_processor(
		&mut self,
		loader: impl Fn(Vec<u8>) -> Result<T, String> + 'static + Send + Sync,
	) {
		self.processor = Some(Box::new(loader));
	}
}

trait FileLoader: Send + Sync + 'static {
	fn load(&self, path: &str, callback: Box<dyn Fn(Result<Vec<u8>, String>)>);
}

struct MiniquadFileLoader;
impl FileLoader for MiniquadFileLoader {
	fn load(&self, path: &str, callback: Box<dyn Fn(Result<Vec<u8>, String>)>) {
		miniquad::fs::load_file(path, move |res| callback(res.map_err(|e| e.to_string())))
	}
}

fn update_assets<T: Component>(mut assets: ResMut<Assets<T>>, mut evt: EventWriter<AssetEvent<T>>) {
	let loaded_files: Option<Vec<LoadedFile<T>>> = {
		let mut loaded_files = assets.pending_loaded_files.lock().unwrap();
		(loaded_files.len() > 0).then(|| loaded_files.drain(..).collect::<Vec<_>>())
	};

	if let Some(files) = loaded_files {
		for file in files.into_iter() {
			if let Some(processor) = &assets.processor {
				let LoadedFile {
					handle,
					path,
					bytes,
				} = file;
				match (processor)(bytes) {
					Ok(value) => assets.insert_asset(&handle, value),
					Err(e) => error!("Failed to process file {}: {}", path, e),
				}
			} else {
				error!("No processor for asset type {:?}", type_name::<T>())
			}
		}
	}

	for handle in assets.pending_created_events.drain(..) {
		evt.send(AssetEvent::Added(handle));
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
			assets.values.remove(&handle.id);
			evt.send(AssetEvent::Removed(handle));
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[derive(Default)]
	struct IntEvents(Vec<i32>);

	fn read(a: &mut App) -> Vec<i32> {
		a.world
			.get_resource::<IntEvents>()
			.unwrap()
			.0
			.iter()
			.copied()
			.collect::<Vec<_>>()
	}

	fn assets(a: &mut App) -> Mut<Assets<i32>> {
		a.world.get_resource_mut::<Assets<i32>>().unwrap()
	}

	#[test]
	fn asset_lifecycle() {
		use super::AssetEvent::*;
		fn log_events(mut evt: EventReader<AssetEvent<i32>>, mut events: ResMut<IntEvents>) {
			let nums = {
				evt.iter()
					.map(|e| {
						(match e {
							Added(h) | Removed(h) => h.id() as i32,
						}) * match e {
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
			.add_system_to_stage(CoreStage::AssetEvents, log_events.system())
			.build();
		let _one = assets(app).add(1);
		{
			let _two = assets(app).add(2);
			let _three = assets(app).add(3);
			app.dispatch_update();
			assert_eq!(read(app), &[1, 2, 3], "frame 1");
		}
		app.dispatch_update();
		app.dispatch_update();
		assert_eq!(read(app), &[1, 2, 3, -2, -3], "frame 2");
	}

	#[test]
	fn file_loading() {
		struct TestLoader;
		impl FileLoader for TestLoader {
			fn load(&self, path: &str, callback: Box<dyn Fn(Result<Vec<u8>, String>)>) {
				if path != "test_file" {
					panic!()
				}
				callback(Ok("contents".into()));
			}
		}

		fn assets(app: &mut App) -> Mut<Assets<String>> {
			app.world.get_resource_mut::<Assets<String>>().unwrap()
		}

		let mut app = App::new()
			.add_asset_type_with_loader::<String, _>(TestLoader {})
			.use_asset_processor(|b| {
				Ok(format!(
					"{} processed",
					std::string::String::from_utf8_lossy(&b)
				))
			})
			.build();
		let handle = assets(&mut app).load("test_file");
		app.dispatch_update();
		assert_eq!(
			*assets(&mut app).get(&handle).unwrap(),
			"contents processed"
		);
	}
}
