mod camera;
mod draw;
mod mesh;
mod shader;
mod texture;

use crate::prelude::*;
use bevy_ecs::component::Component;
pub use camera::{Camera, CameraBundle, ProjectionMatrix, ViewMatrix};
pub use mesh::{Mesh, Vertex};
use miniquad::PipelineParams;
pub use shader::{Shader, ShaderMetaStore};
pub use texture::{Texture, TextureBindings, TextureLoadSettings, TextureProperties};

pub fn plugin(app: &mut AppBuilder) {
	app.add_asset_type::<Texture>()
		.add_asset_type::<Mesh>()
		.add_asset_type::<Shader>()
		.insert_resource(draw::ContextResources::default())
		.insert_resource(texture::TextureLoadSettings::default())
		.insert_resource(shader::ShaderMetaStore::default())
		.use_asset_processor(texture::process_png_texture)
		.use_asset_processor(shader::process_shader_source)
		.add_system_to_stage(CoreStage::AssetEvents, texture::upload_textures.system())
		.add_system_to_stage(CoreStage::AssetEvents, mesh::upload_meshes.system())
		.add_system_to_stage(CoreStage::AssetEvents, shader::upload_shaders.system())
		.add_plugin(camera::plugin);
}

#[derive(Default)]
pub struct RenderSettings {
	pub pipeline: PipelineParams,
}

impl AppBuilder {
	pub fn register_shader_uniforms<T: Component>(&mut self) -> &mut Self {
		self.add_system_to_stage(CoreStage::Render, draw::render::<T>.system())
	}
}
