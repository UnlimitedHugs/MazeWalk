use bevy::{
	asset::{AssetStage, HandleId},
	math::vec2,
	prelude::*,
	reflect::TypeUuid,
	utils::HashMap,
};
use bevy_miniquad::Context;
use miniquad::{
	Bindings, Buffer, BufferLayout, BufferType, Pipeline, Shader as ContextShader, ShaderMeta,
	Texture as ContextTexture, TextureFormat as ContextTextureFormat, TextureParams,
	UniformBlockLayout, UniformDesc, UniformType as ContextUniformType, VertexAttribute,
	VertexFormat,
};

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum RenderStage {
	RenderResource,
	Render,
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_stage_after(
			AssetStage::AssetEvents,
			RenderStage::RenderResource,
			SystemStage::single_threaded(),
		);
		app.add_stage_after(
			CoreStage::PostUpdate,
			RenderStage::Render,
			SystemStage::single_threaded(),
		)
		.add_asset::<Texture>()
		.add_asset::<Mesh>()
		.add_asset::<Shader>()
		.init_resource::<ContextResources>()
		.add_system_to_stage(RenderStage::RenderResource, upload_textures.system())
		.add_system_to_stage(RenderStage::RenderResource, upload_meshes.system())
		.add_system_to_stage(RenderStage::RenderResource, upload_shaders.system())
		.add_system_to_stage(RenderStage::Render, render.system());
	}
}

#[derive(TypeUuid)]
#[uuid = "b028781a-058a-48b7-93cd-61769f97667a"]
pub struct Texture {
	pub data: Vec<u8>,
	pub width: u32,
	pub height: u32,
	pub format: TextureFormat,
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum TextureFormat {
	RGB8,
	RGBA8,
}

impl Into<ContextTextureFormat> for TextureFormat {
	fn into(self) -> ContextTextureFormat {
		match self {
			TextureFormat::RGB8 => ContextTextureFormat::RGB8,
			TextureFormat::RGBA8 => ContextTextureFormat::RGBA8,
		}
	}
}

fn upload_textures(
	textures: Res<Assets<Texture>>,
	mut texture_events: EventReader<AssetEvent<Texture>>,
	mut context: ResMut<Context>,
	mut context_resources: ResMut<ContextResources>,
) {
	for evt in texture_events.iter() {
		if let AssetEvent::Created { handle } = evt {
			if let Some(tex) = textures.get(handle) {
				let overwritten = context_resources
					.textures
					.insert(
						handle.clone(),
						ContextTexture::from_data_and_format(
							&mut context,
							&tex.data,
							TextureParams {
								format: tex.format.into(),
								width: tex.width,
								height: tex.height,
								..Default::default()
							},
						),
					)
					.is_some();
				if overwritten {
					panic!("uploading duplicate texture");
				}
			}
		}
	}
}

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

fn upload_meshes(
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

fn upload_shaders(
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

#[derive(Default)]
struct ContextResources {
	textures: HashMap<Handle<Texture>, ContextTexture>,
	mesh_buffers: HashMap<Handle<Mesh>, MeshBufferSet>,
	pipelines: HashMap<Handle<Shader>, Pipeline>,
}

struct MeshBufferSet {
	vertex: Buffer,
	index: Buffer,
}

fn render(
	mut ctx: ResMut<Context>,
	resources: Res<ContextResources>,
	query: Query<(
		&GlobalTransform,
		&Handle<Mesh>,
		&Handle<Texture>,
		&Handle<Shader>,
	)>,
) {
	let mut grouped_by_shader = query.iter().collect::<Vec<_>>();
	grouped_by_shader.sort_by(|a, b| a.3.id.cmp(&b.3.id));

	ctx.begin_default_pass(Default::default());
	let mut current_shader: Option<HandleId> = None;

	for (transform, mesh_handle, texture_handle, shader_handle) in grouped_by_shader.into_iter() {
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
					ctx.apply_uniforms(&(transform.translation.x, transform.translation.y));
					ctx.draw(0, mesh.index.size() as i32, 1);
				}
			}
		}
	}

	ctx.end_render_pass();
	ctx.commit_frame();
}
