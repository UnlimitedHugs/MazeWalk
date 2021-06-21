use bevy::prelude::*;
use bevy_miniquad::{Context};
use miniquad::*;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum RenderStage {
	Render,
}

pub struct RenderingPlugin;

impl Plugin for RenderingPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.add_stage_after(
			CoreStage::PostUpdate,
			RenderStage::Render,
			SystemStage::single_threaded(),
		);
		app.add_startup_system(initialize.system());
		app.add_system_to_stage(RenderStage::Render, render.system());
	}
}

struct DemoAssets {
	pipeline: Pipeline,
	bindings: Bindings,
}

fn initialize(mut commands: Commands, mut ctx: ResMut<Context>) {
	#[rustfmt::skip]
	let vertices: [Vertex; 4] = [
		Vertex { pos : Vec2 { x: -0.5, y: -0.5 }, uv: Vec2 { x: 0., y: 0. } },
		Vertex { pos : Vec2 { x:  0.5, y: -0.5 }, uv: Vec2 { x: 1., y: 0. } },
		Vertex { pos : Vec2 { x:  0.5, y:  0.5 }, uv: Vec2 { x: 1., y: 1. } },
		Vertex { pos : Vec2 { x: -0.5, y:  0.5 }, uv: Vec2 { x: 0., y: 1. } },
	];
	let vertex_buffer = Buffer::immutable(&mut ctx, BufferType::VertexBuffer, &vertices);

	let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];
	let index_buffer = Buffer::immutable(&mut ctx, BufferType::IndexBuffer, &indices);

	let pixels: [u8; 4 * 4 * 4] = [
		0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
		0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
		0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
		0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
		0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
	];
	let texture = Texture::from_rgba8(&mut ctx, 4, 4, &pixels);

	let bindings = Bindings {
		vertex_buffers: vec![vertex_buffer],
		index_buffer: index_buffer,
		images: vec![texture],
	};

	let shader = Shader::new(&mut ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

	let pipeline = Pipeline::new(
		&mut ctx,
		&[BufferLayout::default()],
		&[
			VertexAttribute::new("pos", VertexFormat::Float2),
			VertexAttribute::new("uv", VertexFormat::Float2),
		],
		shader,
	);

	commands.insert_resource(DemoAssets {pipeline, bindings})
}

fn render(mut ctx: ResMut<Context>, assets: Res<DemoAssets>) {
	let t = miniquad::date::now();

	ctx.begin_default_pass(Default::default());

	ctx.apply_pipeline(&assets.pipeline);
	ctx.apply_bindings(&assets.bindings);
	for i in 0..10 {
		let t = t + i as f64 * 0.3;

		ctx.apply_uniforms(&shader::Uniforms {
			offset: (t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5),
		});
		ctx.draw(0, 6, 1);
	}
	ctx.end_render_pass();

	ctx.commit_frame();
}

#[repr(C)]
struct Vec2 {
	x: f32,
	y: f32,
}
#[repr(C)]
struct Vertex {
	pos: Vec2,
	uv: Vec2,
}

mod shader {
	use miniquad::*;

	pub const VERTEX: &str = r#"#version 100
    attribute vec2 pos;
    attribute vec2 uv;

    uniform vec2 offset;

    varying lowp vec2 texcoord;

    void main() {
        gl_Position = vec4(pos + offset, 0, 1);
        texcoord = uv;
    }"#;

	pub const FRAGMENT: &str = r#"#version 100
    varying lowp vec2 texcoord;

    uniform sampler2D tex;

    void main() {
        gl_FragColor = texture2D(tex, texcoord);
    }"#;

	pub fn meta() -> ShaderMeta {
		ShaderMeta {
			images: vec!["tex".to_string()],
			uniforms: UniformBlockLayout {
				uniforms: vec![UniformDesc::new("offset", UniformType::Float2)],
			},
		}
	}

	#[repr(C)]
	pub struct Uniforms {
		pub offset: (f32, f32),
	}
}
