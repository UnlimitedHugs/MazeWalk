use crate::{
	app::*,
	assets::{AssetEvent, Assets},
};

use super::draw::{ContextResources, MeshBufferSet};
use glam::{Mat4, Vec2, Vec3};
use legion::system;
use miniquad::{Buffer, BufferType, Context, VertexAttribute, VertexFormat};

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

#[derive(Clone)]
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

#[system]
pub fn upload_meshes(
	#[resource] meshes: &mut Assets<Mesh>,
	#[resource] mesh_events: &Event<AssetEvent<Mesh>>,
	#[resource] context: &mut Context,
	#[resource] context_resources: &mut ContextResources,
) {
	for evt in mesh_events.iter() {
		if let AssetEvent::Added(handle) = evt {
			if let Some(mesh) = meshes.get(handle) {
				let overwritten = context_resources
					.mesh_buffers
					.insert(
						handle.id(),
						MeshBufferSet {
							vertex: Buffer::immutable(
								context,
								BufferType::VertexBuffer,
								&mesh.vertices,
							),
							index: Buffer::immutable(
								context,
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
