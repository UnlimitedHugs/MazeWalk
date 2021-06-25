use super::draw::ContextResources;
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_miniquad::Context;
use miniquad::{Texture as ContextTexture, TextureFormat, TextureParams};

#[derive(TypeUuid)]
#[uuid = "b028781a-058a-48b7-93cd-61769f97667a"]
pub struct Texture {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub format: TextureFormat,
}

pub fn upload_textures(
	textures: Res<Assets<Texture>>,
	mut texture_events: EventReader<AssetEvent<Texture>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
) {
	for evt in texture_events.iter() {
		if let AssetEvent::Created { handle } = evt {
			if let Some(tex) = textures.get(handle) {
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
								..Default::default()
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
