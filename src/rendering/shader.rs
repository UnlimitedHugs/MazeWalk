use std::{collections::HashMap, str};

use super::{draw::ContextResources, mesh::Vertex, RenderSettings};
use crate::prelude::*;
use miniquad::{
	BufferLayout, Context, Pipeline, PipelineParams, Shader as ContextShader, ShaderMeta,
	UniformBlockLayout, UniformDesc, UniformType,
};

pub struct Shader {
	pub vertex: String,
	pub fragment: String,
}

impl Shader {
	pub fn new(vertex: &str, fragment: &str) -> Self {
		Self {
			vertex: vertex.to_string(),
			fragment: fragment.to_string(),
		}
	}
}

#[derive(Default)]
pub struct ShaderMetaStore(HashMap<HandleId, ShaderMetadata>);
impl ShaderMetaStore {
	pub fn set(
		&mut self,
		for_shader: &Handle<Shader>,
		textures: &[&str],
		uniforms: &[(&str, UniformType)],
	) {
		self.0.insert(
			for_shader.id(),
			ShaderMetadata {
				textures: textures.into_iter().map(|s| s.to_string()).collect(),
				uniforms: uniforms
					.into_iter()
					.map(|t| (t.0.to_string(), t.1))
					.collect(),
			},
		);
	}
}

struct ShaderMetadata {
	textures: Vec<String>,
	uniforms: Vec<(String, UniformType)>,
}

impl From<&ShaderMetadata> for ShaderMeta {
	fn from(m: &ShaderMetadata) -> Self {
		ShaderMeta {
			images: m.textures.clone(),
			uniforms: UniformBlockLayout {
				uniforms: m
					.uniforms
					.iter()
					.map(|u| UniformDesc::new(&u.0, u.1))
					.collect(),
			},
		}
	}
}

pub fn upload_shaders(
	shaders: Res<Assets<Shader>>,
	mut shader_events: EventReader<AssetEvent<Shader>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
	settings: Option<Res<RenderSettings>>,
	meta_store: Res<ShaderMetaStore>,
) {
	let mut register_shader = |handle: &Handle<Shader>, ctx: &mut ContextResources| {
		let shader = shaders.get(handle).expect("resolve shader asset");
		let shader = ContextShader::new(
			&mut context,
			&shader.vertex,
			&shader.fragment,
			meta_store
				.0
				.get(&handle.id())
				.unwrap_or_else(|| panic!("shader requires metadata: {:?}", handle.id()))
				.into(),
		);
		let pipeline_params = match settings {
			Some(ref res) => PipelineParams { ..res.pipeline },
			None => Default::default(),
		};
		match shader {
			Ok(shader) => {
				let overwritten = ctx
					.pipelines
					.insert(
						handle.id(),
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
			Err(e) => eprintln!("Shader compilation error: {}", e),
		}
	};
	fn discard_shader(handle: &Handle<Shader>, ctx: &mut ContextResources) {
		ctx.pipelines.remove(&handle.id());
	}

	for evt in shader_events.iter() {
		match evt {
			AssetEvent::Added(handle) => register_shader(handle, &mut context_resources),
			AssetEvent::Removed(handle) => discard_shader(handle, &mut context_resources),
		}
	}
}

pub fn process_shader_source(bytes: Vec<u8>) -> Result<Shader, String> {
	if let Ok(contents) = str::from_utf8(bytes.as_slice()) {
		if !contents.starts_with("#version") {
			return Err("expected version directive".to_string());
		}
		if let Some(version_newline_pos) = contents.find('\n') {
			let vertex = {
				let mut v = contents.to_string();
				v.insert_str(version_newline_pos + 1, "#define VERTEX\n");
				v
			};
			let fragment = contents;
			Ok(Shader::new(&vertex, fragment))
		} else {
			Err("expected newline after version directive".to_string())
		}
	} else {
		Err("failed to read shader utf8".to_string())
	}
}