use super::{RenderSettings, draw::ContextResources, mesh::Vertex};
use bevy::{prelude::*, reflect::TypeUuid};
use bevy_miniquad::Context;
use miniquad::{
	BufferLayout, Pipeline, PipelineParams, Shader as ContextShader, ShaderMeta,
	UniformBlockLayout, UniformDesc, UniformType,
};

#[derive(TypeUuid)]
#[uuid = "e91c0308-4bf9-42eb-95ef-0b95a24f9a72"]
pub struct Shader {
	pub vertex: String,
	pub fragment: String,
	pub meta: ShaderMetadata,
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
					.map(|u| UniformDesc::new(&u.0, u.1))
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
	settings: Option<Res<RenderSettings>>,
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
				let pipeline_params = match settings {
					Some(ref res) => PipelineParams {
						..res.pipeline
					},
					None => Default::default(),
				};
				let overwritten = context_resources
					.pipelines
					.insert(
						handle.clone(),
						Pipeline::with_params(
							&mut context,
							&[BufferLayout::default()],
							&Vertex::attributes(),
							shader,
							pipeline_params,
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
