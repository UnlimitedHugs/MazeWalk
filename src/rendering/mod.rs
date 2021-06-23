mod draw;
mod mesh;
mod shader;
mod texture;

pub use mesh::{Mesh, Vertex};
pub use shader::{Shader, ShaderMetadata, UniformType};
pub use texture::{Texture, TextureFormat};

use bevy::{asset::AssetStage, ecs::component::Component, prelude::*};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum RenderStage {
	RenderResource,
	Render,
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_stage_after(
			AssetStage::AssetEvents,
			RenderStage::RenderResource,
			SystemStage::single_threaded(),
		);
		app.add_stage_after(
			CoreStage::PostUpdate,
			RenderStage::Render,
			SystemStage::single_threaded(),
		)
		.add_asset::<Texture>()
		.add_asset::<Mesh>()
		.add_asset::<Shader>()
		.init_resource::<draw::ContextResources>()
		.add_system_set_to_stage(
			RenderStage::RenderResource,
			SystemSet::new()
				.with_system(texture::upload_textures.system())
				.with_system(mesh::upload_meshes.system())
				.with_system(shader::upload_shaders.system()),
		);
	}
}

pub trait AppExtensions {
	fn register_shader_uniforms<T: Component>(&mut self) -> &mut Self;
}
impl AppExtensions for AppBuilder {
	fn register_shader_uniforms<T: Component>(&mut self) -> &mut Self {
		self.add_system_to_stage(RenderStage::Render, draw::render::<T>.system())
	}
}

