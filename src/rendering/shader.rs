use super::draw::ContextResources;
use super::mesh::Vertex;
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_miniquad::Context;
use miniquad::{
	BufferLayout, Pipeline, Shader as ContextShader, ShaderMeta, UniformBlockLayout, UniformDesc,
	UniformType as ContextUniformType,
};

#[derive(TypeUuid)]
#[uuid = "e91c0308-4bf9-42eb-95ef-0b95a24f9a72"]
pub struct Shader {
	pub vertex: String,
	pub fragment: String,
	pub meta: ShaderMetadata,
}

#[derive(Clone, Copy)]
pub enum UniformType {
	Float1,
	Float2,
	Float3,
	Float4,
}

impl From<UniformType> for ContextUniformType {
	fn from(t: UniformType) -> Self {
		match t {
			UniformType::Float1 => ContextUniformType::Float1,
			UniformType::Float2 => ContextUniformType::Float2,
			UniformType::Float3 => ContextUniformType::Float3,
			UniformType::Float4 => ContextUniformType::Float4,
		}
	}
}

impl Shader {
	pub fn new(
		vertex: &str,
		fragment: &str,
		textures: &[&str],
		uniforms: &[(&str, UniformType)],
	) -> Self {
		Self {
			vertex: vertex.to_string(),
			fragment: fragment.to_string(),
			meta: ShaderMetadata {
				textures: textures.iter().map(|s| s.to_string()).collect(),
				uniforms: uniforms
					.into_iter()
					.map(|t| (t.0.to_string(), t.1))
					.collect(),
			},
		}
	}
}

pub struct ShaderMetadata {
	pub textures: Vec<String>,
	pub uniforms: Vec<(String, UniformType)>,
}

impl ShaderMetadata {
	fn get_meta(&self) -> ShaderMeta {
		ShaderMeta {
			images: self.textures.clone(),
			uniforms: UniformBlockLayout {
				uniforms: self
					.uniforms
					.iter()
					.map(|u| UniformDesc::new(&u.0, u.1.into()))
					.collect(),
			},
		}
	}
}

pub fn upload_shaders(
	meshes: Res<Assets<Shader>>,
	mut shader_events: EventReader<AssetEvent<Shader>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
) {
	for evt in shader_events.iter() {
		if let AssetEvent::Created { handle } = evt {
			if let Some(shader) = meshes.get(handle) {
				let shader = ContextShader::new(
					&mut context,
					&shader.vertex,
					&shader.fragment,
					shader.meta.get_meta(),
				)
				.unwrap();
				let overwritten = context_resources
					.pipelines
					.insert(
						handle.clone(),
						Pipeline::new(
							&mut context,
							&[BufferLayout::default()],
							&Vertex::attributes(),
							shader,
						),
					)
					.is_some();
				if overwritten {
					panic!("uploading duplicate shader");
				}
			}
		}
	}
}
