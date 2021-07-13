use super::{mesh::Mesh, shader::Shader, texture::Texture, TextureBindings};
use bevy::ecs::component::Component;
use bevy::{asset::HandleId, prelude::*, utils::HashMap};
use bevy_miniquad::Context;
use miniquad::{Bindings, Buffer, PassAction, Pipeline, Texture as ContextTexture};

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
		&Handle<Shader>,
		Option<&TextureBindings>,
		&Uniforms,
	)>,
) {
	let mut grouped_by_shader = query.iter().collect::<Vec<_>>();
	grouped_by_shader.sort_by(|a, b| a.1.id.cmp(&b.1.id));

	ctx.begin_default_pass(PassAction::Clear {
		color: Some((0.2, 0.2, 0.2, 1.0)),
		depth: Some(1.),
		stencil: None,
	});
	let mut current_shader: Option<HandleId> = None;
	for (mesh_handle, shader_handle, optional_textures, uniforms) in grouped_by_shader.into_iter() {
		if let (Some(mesh), Some(pipeline)) = (
			resources.mesh_buffers.get(mesh_handle),
			resources.pipelines.get(shader_handle),
		) {
			let images = if let Some(TextureBindings(bindings)) = optional_textures {
				let resolved = bindings
					.iter()
					.filter_map(|h| resources.textures.get(h))
					.copied()
					.collect::<Vec<_>>();
				if resolved.len() < bindings.len() {
					// not all textures loaded, skip drawing object
					continue;
				}
				resolved
			} else {
				vec![]
			};

			if current_shader.is_none() || current_shader != Some(shader_handle.id) {
				current_shader = Some(shader_handle.id);
				ctx.apply_pipeline(&pipeline);
			}
			ctx.apply_bindings(&Bindings {
				vertex_buffers: vec![mesh.vertex],
				index_buffer: mesh.index,
				images,
			});
			ctx.apply_uniforms(uniforms);
			ctx.draw(0, mesh.index.size() as i32, 1);
		}
	}

	ctx.end_render_pass();
	ctx.commit_frame();
}
