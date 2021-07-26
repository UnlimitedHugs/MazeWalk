use super::{GameState, Material};
use bevy::{
	asset::{AssetLoader, AssetStage, BoxedFuture, LoadContext, LoadedAsset},
	prelude::*,
	reflect::TypeUuid,
};
use serde_derive::Deserialize;
use serde_yaml;

#[derive(Deserialize, TypeUuid, Clone, PartialEq)]
#[uuid = "5b21bd2a-d3b5-4dc8-beb7-435cb1b0e3d8"]
pub struct Tweaks {
	pub ambient_light_intensity: f32,
	pub ceiling_material: Material,
	pub wall_material: Material,
	pub floor_material: Material,
}

pub struct TweaksPlugin;
impl Plugin for TweaksPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_asset::<Tweaks>()
			.init_asset_loader::<TweaksLoader>()
			.add_system_to_stage(AssetStage::AssetEvents, tweaks_to_resource.system());
	}
}

pub fn wait_for_tweaks_ready(mut state: ResMut<State<GameState>>, tweaks: Option<Res<Tweaks>>) {
	if tweaks.is_some() {
		state.replace(GameState::Play).unwrap();
	}
}

pub fn restart_on_tweaks_changed(
	mut state: ResMut<State<GameState>>,
	tweaks: Res<Tweaks>,
	mut initial_ignored: Local<bool>,
) {
	if tweaks.is_changed() {
		if !*initial_ignored {
			*initial_ignored = true;
			return;
		}
		state.replace(GameState::Preload).unwrap();
	}
}

fn tweaks_to_resource(
	mut cmd: Commands,
	mut tweaks_events: EventReader<AssetEvent<Tweaks>>,
	tweaks_res: Option<Res<Tweaks>>,
	tweaks_assets: Res<Assets<Tweaks>>,
) {
	for e in tweaks_events.iter() {
		match e {
			AssetEvent::Created { handle } | AssetEvent::Modified { handle } => {
				let tweaks = tweaks_assets.get(handle).expect("tweaks handle resolve");
				let changed = if let Some(ref res) = tweaks_res {
					!res.eq(tweaks)
				} else {
					true
				};
				if changed {
					cmd.insert_resource(tweaks.clone());
				}
			}
			_ => {}
		}
	}
}

#[derive(Default)]
struct TweaksLoader;

impl AssetLoader for TweaksLoader {
	fn load<'a>(
		&'a self,
		bytes: &'a [u8],
		load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
		Box::pin(async move {
			let tweaks = serde_yaml::from_slice::<Tweaks>(bytes)?;
			load_context.set_default_asset(LoadedAsset::new(tweaks));
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["tweaks.yml", "yml"]
	}
}
