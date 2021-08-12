use std::collections::HashMap;

use super::draw::ContextResources;
use crate::prelude::*;
use miniquad::{
	Context, FilterMode, Texture as ContextTexture, TextureFormat, TextureParams, TextureWrap,
};
use png::{ColorType, Decoder};

pub struct Texture {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub format: TextureFormat,
}

#[derive(Clone)]
pub struct TextureBindings(pub Vec<Handle<Texture>>);

#[derive(Default)]
pub struct TextureLoadSettings {
	per_asset: HashMap<HandleId, TextureProperties>,
	defaults: TextureProperties,
}
impl TextureLoadSettings {
	pub fn set_defaults(&mut self, props: TextureProperties) {
		self.defaults = props;
	}
}

#[derive(Clone, Copy)]
pub struct TextureProperties {
	pub wrap: TextureWrap,
	pub filter: FilterMode,
	pub anisotropy: f32,
}
impl Default for TextureProperties {
	fn default() -> Self {
		Self {
			wrap: TextureWrap::Clamp,
			filter: FilterMode::Linear,
			anisotropy: 0.0,
		}
	}
}

pub fn upload_textures(
	textures: Res<Assets<Texture>>,
	mut texture_events: EventReader<AssetEvent<Texture>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
	load_settings: Res<TextureLoadSettings>,
) {
	for evt in texture_events.iter() {
		if let AssetEvent::Added(handle) = evt {
			if let Some(tex) = textures.get(handle) {
				let TextureProperties {
					wrap,
					filter,
					anisotropy,
				} = load_settings
					.per_asset
					.get(&handle.id())
					.unwrap_or_else(|| &load_settings.defaults);
				let overwritten = context_resources
					.textures
					.insert(
						handle.id(),
						ContextTexture::from_data_and_format(
							&mut context,
							&tex.data,
							TextureParams {
								format: tex.format,
								width: tex.width,
								height: tex.height,
								wrap: *wrap,
								filter: *filter,
								anisotropy: *anisotropy,
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

pub fn process_png_texture(bytes: Vec<u8>) -> Result<Texture, String> {
	let decoder = Decoder::new(bytes.as_slice());
	let (png_info, mut png_reader) = decoder.read_info().unwrap();
	let mut data = vec![0; png_info.buffer_size()];
	png_reader.next_frame(&mut data).unwrap();
	let format = match png_info.color_type {
		ColorType::RGB => TextureFormat::RGB8,
		ColorType::RGBA => TextureFormat::RGBA8,
		t => return Err(format!("Unsupported PNG format: {:?}", t)),
	};
	let tex = Texture {
		data,
		width: png_info.width,
		height: png_info.height,
		format,
	};

	Ok(tex)
}