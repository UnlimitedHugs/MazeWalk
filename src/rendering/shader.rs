use std::{collections::HashMap, str};

use super::{draw::ContextResources, mesh::Vertex, RenderSettings};
use anyhow::{bail, Context as _};
use bevy::{
	asset::{AssetLoader, BoxedFuture, HandleId, LoadContext, LoadedAsset},
	prelude::*,
	reflect::TypeUuid,
};
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
			for_shader.id,
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
	meshes: Res<Assets<Shader>>,
	mut shader_events: EventReader<AssetEvent<Shader>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
	settings: Option<Res<RenderSettings>>,
	meta_store: Res<ShaderMetaStore>,
) {
	let mut register_shader = |handle: &Handle<Shader>, ctx: &mut ContextResources| {
		let shader = meshes.get(handle).expect("resolve shader asset");
		let shader = ContextShader::new(
			&mut context,
			&shader.vertex,
			&shader.fragment,
			meta_store
				.0
				.get(&handle.id)
				.unwrap_or_else(|| panic!("shader requires metadata: {:?}", handle))
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
			Err(e) => eprintln!("Shader compilation error: {}", e),
		}
	};
	fn discard_shader(handle: &Handle<Shader>, ctx: &mut ContextResources) {
		ctx.pipelines.remove(handle);
	}

	for evt in shader_events.iter() {
		match evt {
			AssetEvent::Created { handle } => register_shader(handle, &mut context_resources),
			AssetEvent::Modified { handle } => {
				discard_shader(handle, &mut context_resources);
				register_shader(handle, &mut context_resources);
			}
			AssetEvent::Removed { handle } => discard_shader(handle, &mut context_resources),
		}
	}
}

#[derive(Default)]
pub struct ShaderLoader;

impl AssetLoader for ShaderLoader {
	fn load<'a>(
		&'a self,
		bytes: &'a [u8],
		load_context: &'a mut LoadContext,
	) -> BoxedFuture<'a, Result<(), anyhow::Error>> {
		Box::pin(async move {
			let contents = str::from_utf8(bytes).with_context(|| "read shader utf8")?;
			if contents.len() == 0 {
				// asset loader bug?
				return Ok(())
			}
			if !contents.starts_with("#version") {
				bail!("expected version directive")
			}
			let version_newline_pos = contents
				.find('\n')
				.with_context(|| "expected newline after version directive")?;
			let vertex = {
				let mut v = contents.to_string();
				v.insert_str(version_newline_pos + 1, "#define VERTEX\n");
				v
			};
			let fragment = contents;
			load_context.set_default_asset(LoadedAsset::new(Shader::new(&vertex, fragment)));
			Ok(())
		})
	}

	fn extensions(&self) -> &[&str] {
		&["glsl"]
	}
}
