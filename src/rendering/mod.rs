mod camera;
mod draw;
mod mesh;
mod shader;
mod texture;

use std::fmt::Debug;

use bevy_miniquad::Context;
pub use camera::{Camera, CameraBundle, ProjectionMatrix, ViewMatrix};
pub use mesh::{Mesh, Vertex};
use miniquad::PipelineParams;
pub use shader::{Shader, ShaderMetadata};
pub use texture::{Texture, TextureBindings};

use bevy::{asset::AssetStage, ecs::component::Component, prelude::*};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum RenderStage {
	RenderResource,
	PreRender,
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
			RenderStage::PreRender,
			SystemStage::parallel(),
		);
		app.add_stage_after(
			RenderStage::PreRender,
			RenderStage::Render,
			SystemStage::single_threaded(),
		)
		.add_asset::<Texture>()
		.add_asset::<Mesh>()
		.add_asset::<Shader>()
		.init_resource::<draw::ContextResources>()
		.init_asset_loader::<texture::PngTextureLoader>()
		.add_system(capture_mouse.system())
		.add_system_set_to_stage(
			RenderStage::RenderResource,
			SystemSet::new()
				.with_system(texture::upload_textures.system())
				.with_system(mesh::upload_meshes.system())
				.with_system(shader::upload_shaders.system()),
		)
		.add_plugin(camera::CameraPlugin);
	}
}

#[derive(Default)]
pub struct RenderSettings {
	pub pipeline: PipelineParams,
	pub capture_mouse: bool,
}

fn capture_mouse(ctx: Res<Context>, settings: Res<RenderSettings>) {
	ctx.set_cursor_grab(settings.capture_mouse);
	ctx.show_mouse(!settings.capture_mouse);
}

pub trait AppExtensions {
	fn register_shader_uniforms<T: Component>(&mut self) -> &mut Self;
}
impl AppExtensions for AppBuilder {
	fn register_shader_uniforms<T: Component>(&mut self) -> &mut Self {
		self.add_system_to_stage(RenderStage::Render, draw::render::<T>.system())
	}
}
