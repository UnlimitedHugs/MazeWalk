use crate::{app::*, assets::*, backend::Time, rendering::*, utils::*};
use glam::{vec2, Vec2};
use legion::{system, systems::CommandBuffer, world::SubWorld, Query};
use miniquad::TextureFormat;

pub fn plugin(app: &mut AppBuilder) {
	app.register_shader_uniforms::<QuadUniforms>()
		.insert_resource(RenderSettings::default())
		.add_startup_system(spawn_quads_system())
		.add_system(update_quads_system());
}

#[system]
fn spawn_quads(
	commands: &mut CommandBuffer,
	#[resource] meshes: &mut Assets<Mesh>,
	#[resource] textures: &mut Assets<Texture>,
	#[resource] shaders: &mut Assets<Shader>,
	#[resource] shader_meta: &mut ShaderMetaStore,
) {
	let mesh = meshes.add(Quad::new(vec2(1.0, 1.0)).into());
	let texture = textures.add(Texture {
		data: vec![
			0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00,
			0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF,
			0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF,
			0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
			0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
		],
		width: 4,
		height: 4,
		format: TextureFormat::RGBA8,
	});
	let shader = shaders.add(Shader::new(shader::VERTEX, shader::FRAGMENT));

	shader_meta.set(&shader, &shader::TEXTURES, &shader::UNIFORMS);

	for i in 0..10 {
		commands.push((
			DemoQuad { index: i },
			mesh.clone(),
			shader.clone(),
			QuadUniforms::default(),
			TextureBindings(vec![texture.clone()]),
		));
	}
}

struct DemoQuad {
	index: u32,
}

#[repr(C)]
#[derive(Default, Debug)]
struct QuadUniforms {
	position: Vec2,
}

#[system]
fn update_quads(
	query: &mut Query<(&DemoQuad, &mut QuadUniforms)>,
	#[resource] time: &Time,
	w: &mut SubWorld,
) {
	for (quad, mut uniforms) in query.iter_mut(w) {
		let t = time.seconds_since_startup() + quad.index as f64 * 0.3;
		uniforms.position = vec2(t.sin() as f32 * 0.5, (t * 3.).cos() as f32 * 0.5);
	}
}

mod shader {
	use miniquad::UniformType;

	pub const VERTEX: &str = r#"#version 100
	attribute vec3 pos;
	attribute vec3 normal;
	attribute vec2 uv;

	uniform vec2 offset;

	varying lowp vec2 texcoord;

	void main() {
		gl_Position = vec4(pos.xy + offset, 0, 1);
		texcoord = uv;
	}"#;

	pub const FRAGMENT: &str = r#"#version 100
	varying lowp vec2 texcoord;

	uniform sampler2D tex;

	void main() {
		gl_FragColor = texture2D(tex, texcoord);
	}"#;

	pub const TEXTURES: [&str; 1] = ["tex"];
	pub const UNIFORMS: [(&str, UniformType); 1] = [("offset", UniformType::Float2)];
}
