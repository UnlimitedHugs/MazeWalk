use super::draw::ContextResources;
use anyhow::anyhow;
use bevy::{
	asset::{AssetLoader, BoxedFuture, LoadContext, LoadedAsset},
	prelude::*,
	reflect::TypeUuid,
	utils::HashMap,
};
use bevy_miniquad::Context;
use miniquad::{FilterMode, Texture as ContextTexture, TextureFormat, TextureParams, TextureWrap};
use png::{ColorType, Decoder};

#[derive(TypeUuid)]
#[uuid = "b028781a-058a-48b7-93cd-61769f97667a"]
pub struct Texture {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub format: TextureFormat,
}

#[derive(Clone)]
pub struct TextureBindings(pub Vec<Handle<Texture>>);

#[derive(Default)]
pub struct TextureLoadSettings(HashMap<Handle<Texture>, TextureProperties>);
impl TextureLoadSettings {
	pub fn add(&mut self, for_tex: Handle<Texture>, params: TextureProperties) {
		self.0.insert(for_tex, params);
	}
}

#[derive(Clone, Copy)]
pub struct TextureProperties {
	pub wrap: TextureWrap,
	pub filter: FilterMode,
}
impl Default for TextureProperties {
	fn default() -> Self {
		Self {
			wrap: TextureWrap::Clamp,
			filter: FilterMode::Linear,
		}
	}
}

pub fn upload_textures(
	textures: Res<Assets<Texture>>,
	mut texture_events: EventReader<AssetEvent<Texture>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
	mut load_settings: ResMut<TextureLoadSettings>,
) {
	for evt in texture_events.iter() {
		if let AssetEvent::Created { handle } = evt {
			if let Some(tex) = textures.get(handle) {
				let settings = load_settings.0.remove(handle).unwrap_or_default();
				let overwritten = context_resources
					.textures
					.insert(
						handle.clone(),
						ContextTexture::from_data_and_format(
							&mut context,
							&tex.data,
							TextureParams {
								format: tex.format,
								width: tex.width,
								height: tex.height,
								wrap: settings.wrap,
								filter: settings.filter,
							},
						),
					)
					.is_some();
				if overwritten {
					panic!("uploading duplicate texture");
				}
			}
		}
	}
}

#[derive(Default)]
pub struct PngTextureLoader;

impl AssetLoader for PngTextureLoader {
	fn load<'a>(
		&'a self,
		bytes: &'a [u8],
		load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
		Box::pin(async move {
			let decoder = Decoder::new(bytes);
			let (png_info, mut png_reader) = decoder.read_info().unwrap();
			let mut data = vec![0; png_info.buffer_size()];
			png_reader.next_frame(&mut data).unwrap();
			let format = match png_info.color_type {
				ColorType::RGB => Ok(TextureFormat::RGB8),
				ColorType::RGBA => Ok(TextureFormat::RGBA8),
				t => Err(anyhow!("Unsupported PNG format: {:?}", t)),
			}?;
			let tex = Texture {
				data,
				width: png_info.width,
				height: png_info.height,
				format,
			};

			load_context.set_default_asset(LoadedAsset::new(tex));
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["png"]
	}
}
