use super::mesh::Mesh;
use super::shader::Shader;
use super::texture::Texture;
use bevy::ecs::component::Component;
use bevy::{asset::HandleId, prelude::*, utils::HashMap};
use bevy_miniquad::Context;
use miniquad::{Bindings, Buffer, Pipeline, Texture as ContextTexture};

#[derive(Default)]
pub struct ContextResources {
	pub textures: HashMap<Handle<Texture>, ContextTexture>,
	pub mesh_buffers: HashMap<Handle<Mesh>, MeshBufferSet>,
	pub pipelines: HashMap<Handle<Shader>, Pipeline>,
}

pub struct MeshBufferSet {
	pub vertex: Buffer,
	pub index: Buffer,
}

pub fn render<Uniforms: Component>(
	mut ctx: ResMut<Context>,
	resources: Res<ContextResources>,
	query: Query<(
		&Handle<Mesh>,
		&Handle<Texture>,
		&Handle<Shader>,
		&Uniforms,
	)>,
) {
	let mut grouped_by_shader = query.iter().collect::<Vec<_>>();
	grouped_by_shader.sort_by(|a, b| a.2.id.cmp(&b.2.id));

	ctx.begin_default_pass(Default::default());
	let mut current_shader: Option<HandleId> = None;

	for (mesh_handle, texture_handle, shader_handle, uniforms) in grouped_by_shader.into_iter() {
		if let Some(mesh) = resources.mesh_buffers.get(mesh_handle) {
			if let Some(texture) = resources.textures.get(texture_handle) {
				if let Some(pipeline) = resources.pipelines.get(shader_handle) {
					if current_shader.is_none() || current_shader != Some(shader_handle.id) {
						current_shader = Some(shader_handle.id);
						ctx.apply_pipeline(&pipeline);
					}
					ctx.apply_bindings(&Bindings {
						vertex_buffers: vec![mesh.vertex],
						index_buffer: mesh.index,
						images: vec![*texture],
					});
					ctx.apply_uniforms(uniforms);
					ctx.draw(0, mesh.index.size() as i32, 1);
				}
			}
		}
	}

	ctx.end_render_pass();
	ctx.commit_frame();
}
