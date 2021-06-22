use super::drawing::{ContextResources, MeshBufferSet};
use bevy::{math::vec2, prelude::*, reflect::TypeUuid};
use bevy_miniquad::Context;
use miniquad::{Buffer, BufferType, VertexAttribute, VertexFormat};

#[repr(C)]
pub struct Vertex {
	pub pos: Vec2,
	pub uv: Vec2,
}

impl Vertex {
	pub fn attributes<'a>() -> Vec<VertexAttribute> {
		vec![
			VertexAttribute::new("pos", VertexFormat::Float2),
			VertexAttribute::new("uv", VertexFormat::Float2),
		]
	}

	pub fn new(pos_x: f32, pos_y: f32, uv_x: f32, uv_y: f32) -> Self {
		Self {
			pos: vec2(pos_x, pos_y),
			uv: vec2(uv_x, uv_y),
		}
	}
}

#[derive(TypeUuid)]
#[uuid = "f8d1bdbe-a1ed-41b0-8e45-668e1dcb9899"]
pub struct Mesh {
	vertices: Vec<Vertex>,
	indices: Vec<u16>,
}

pub fn upload_meshes(
	meshes: Res<Assets<Mesh>>,
	mut mesh_events: EventReader<AssetEvent<Mesh>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
) {
	for evt in mesh_events.iter() {
		if let AssetEvent::Created { handle } = evt {
			if let Some(mesh) = meshes.get(handle) {
				let overwritten = context_resources
					.mesh_buffers
					.insert(
						handle.clone(),
						MeshBufferSet {
							vertex: Buffer::immutable(
								&mut context,
								BufferType::VertexBuffer,
								&mesh.vertices,
							),
							index: Buffer::immutable(
								&mut context,
								BufferType::IndexBuffer,
								&mesh.indices,
							),
						},
					)
					.is_some();
				if overwritten {
					panic!("uploading duplicate mesh");
				}
			}
		}
	}
}

impl Mesh {
	pub fn quad(size: f32) -> Self {
		Self {
			vertices: vec![
				Vertex::new(-size, -size, 0., 0.),
				Vertex::new(size, -size, 1., 0.),
				Vertex::new(size, size, 1., 1.),
				Vertex::new(-size, size, 0., 1.),
			],
			indices: vec![0, 1, 2, 0, 2, 3],
		}
	}
}
