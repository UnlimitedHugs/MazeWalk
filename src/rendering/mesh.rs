use super::draw::{ContextResources, MeshBufferSet};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_miniquad::Context;
use miniquad::{Buffer, BufferType, VertexAttribute, VertexFormat};

#[repr(C)]
pub struct Vertex {
	pub pos: Vec3,
	pub normal: Vec3,
	pub uv: Vec2,
}

impl Vertex {
	pub fn attributes<'a>() -> Vec<VertexAttribute> {
		vec![
			VertexAttribute::new("pos", VertexFormat::Float3),
			VertexAttribute::new("normal", VertexFormat::Float3),
			VertexAttribute::new("uv", VertexFormat::Float2),
		]
	}
}

#[derive(TypeUuid)]
#[uuid = "f8d1bdbe-a1ed-41b0-8e45-668e1dcb9899"]
pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u16>,
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
