mod camera;
mod draw;
mod mesh;
mod shader;
mod texture;

use super::app::*;
pub use camera::{Camera, CameraBundle, ProjectionMatrix, ViewMatrix};
use legion::{storage::Component, system};
pub use mesh::{Mesh, Vertex};
use miniquad::{Context, PipelineParams};
pub use shader::{Shader, ShaderMetaStore};
pub use texture::{Texture, TextureBindings, TextureLoadSettings, TextureProperties};

pub fn plugin(app: &mut AppBuilder) {
	app.add_asset_type::<Texture>()
		.add_asset_type::<Mesh>()
		.add_asset_type::<Shader>()
		.insert_resource(draw::ContextResources::default())
		.insert_resource(texture::TextureLoadSettings::default())
		.insert_resource(shader::ShaderMetaStore::default())
		//.init_asset_loader::<texture::PngTextureLoader>()
		//.init_asset_loader::<shader::ShaderLoader>()
		.add_system(capture_mouse_system())
		.add_system_to_stage(texture::upload_textures_system(), Stage::AssetEvents)
		.add_system_to_stage(mesh::upload_meshes_system(), Stage::AssetEvents)
		.add_system_to_stage(shader::upload_shaders_system(), Stage::AssetEvents)
		.add_plugin(camera::plugin);
}

#[derive(Default)]
pub struct RenderSettings {
	pub pipeline: PipelineParams,
	pub capture_mouse: bool,
}

#[system]
fn capture_mouse(#[resource] ctx: &Context, #[resource] settings: &RenderSettings) {
	ctx.set_cursor_grab(settings.capture_mouse);
	ctx.show_mouse(!settings.capture_mouse);
}

impl AppBuilder {
	pub fn register_shader_uniforms<T: Component>(&mut self) -> &mut Self {
		self.add_system_to_stage(draw::render_system::<T>(), Stage::Render)
	}
}
