use super::draw::{ContextResources, MeshBufferSet};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_miniquad::Context;
use miniquad::{Buffer, BufferType, VertexAttribute, VertexFormat};

#[derive(Clone)]
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

	fn transform(&self, mat: Mat4) -> Self {
		Vertex {
			pos: mat.transform_point3(self.pos),
			normal: mat.transform_vector3(self.normal),
			uv: self.uv,
		}
	}
}

#[derive(TypeUuid, Clone)]
#[uuid = "f8d1bdbe-a1ed-41b0-8e45-668e1dcb9899"]
pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u16>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			vertices: vec![],
			indices: vec![],
		}
	}

	pub fn extend_with(&mut self, other: Mesh) {
		let offset = self.vertices.len() as u16;
		self.vertices.extend(other.vertices.into_iter());
		self.indices
			.extend(other.indices.into_iter().map(|i| i + offset));
	}

	pub fn transform(&self, mat: Mat4) -> Mesh {
		Mesh {
			vertices: self.vertices.iter().map(|v| v.transform(mat)).collect(),
			indices: self.indices.clone(),
		}
	}
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
