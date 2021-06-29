use super::{rendering::*, utils::Cube};
use bevy::{
	input::mouse::MouseMotion,
	math::{vec2, vec3},
	prelude::*,
};
use miniquad::{Comparison, CullFace, PipelineParams};
use rand::Rng;

pub struct MazePlugin;
impl Plugin for MazePlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.insert_resource(RenderSettings {
			pipeline: PipelineParams {
				depth_test: Comparison::LessOrEqual,
				depth_write: true,
				cull_face: CullFace::Back,
				..Default::default()
			},
			capture_mouse: true,
		})
		.register_shader_uniforms::<Uniforms>()
		.add_startup_system(build_maze.system())
		.add_system(update_uniforms.system())
		.add_system(camera_input.system().chain(expand_euler_rotation.system()));
	}
}

struct Wall;

fn build_maze(
	mut cmd: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut shaders: ResMut<Assets<Shader>>,
) {
	let cube_mesh = meshes.add(Cube::new(1.0).into());
	let shader = shaders.add(Shader::new(
		shader::VERTEX,
		shader::FRAGMENT,
		&shader::TEXTURES,
		&shader::UNIFORMS,
	));
	let mut rng = rand::thread_rng();

	for x in 0..16 {
		for z in 0..16 {
			if rng.gen_ratio(2, 3) {
				continue;
			}
			let transform = GlobalTransform::from_translation(vec3(x as f32, 0., -z as f32));
			cmd.spawn_bundle((
				Wall,
				transform,
				cube_mesh.clone(),
				shader.clone(),
				Uniforms {
					model: transform.compute_matrix(),
					..Default::default()
				},
			));
		}
	}

	cmd.spawn_bundle(CameraBundle {
		transform: GlobalTransform::from_translation(vec3(0., 0., 4.)),
		camera: Camera {
			field_of_view: 75.0,
			clipping_distance: 0.1..100.,
		},
		..Default::default()
	})
	.insert(RotationEuler::default());
}

#[derive(Default)]
struct RotationEuler(Vec2);

fn camera_input(
	mut q: Query<(&mut GlobalTransform, &mut RotationEuler), With<Camera>>,
	key: Res<Input<KeyCode>>,
	mut mouse_motion: EventReader<MouseMotion>,
	t: Res<Time>,
) {
	let mut movement = Vec3::ZERO;
	if key.pressed(KeyCode::W) {
		movement += vec3(0., 0., -1.0);
	}
	if key.pressed(KeyCode::S) {
		movement += vec3(0., 0., 1.0);
	}
	if key.pressed(KeyCode::A) {
		movement += vec3(-1., 0., 0.);
	}
	if key.pressed(KeyCode::D) {
		movement += vec3(1., 0., 0.);
	}

	let (mut transform, mut euler) = q.single_mut().unwrap();
	let mouse_sensitivity = 0.008f32;
	let pitch_limit = 90.0f32.to_radians() * 0.99;
	for MouseMotion { delta } in mouse_motion.iter() {
		euler.0 = vec2(
			euler.0.x - delta.x * mouse_sensitivity,
			(euler.0.y - delta.y * mouse_sensitivity).clamp(-pitch_limit, pitch_limit),
		);
	}
	if movement != Vec3::ZERO {
		let view_relative =
			Quat::from_rotation_ypr(euler.0.x, 0., 0.) * (movement * 3. * t.delta_seconds());
		transform.translation += view_relative;
	}
}

fn expand_euler_rotation(
	mut q: Query<(&mut GlobalTransform, &RotationEuler), Changed<RotationEuler>>,
) {
	// separate system to handle startup value
	for (mut tx, RotationEuler(r)) in q.iter_mut() {
		tx.rotation = Quat::from_rotation_ypr(r.x, r.y, 0.);
	}
}

fn update_uniforms(
	mut q: QuerySet<(
		Query<(&GlobalTransform, &ViewMatrix, &ProjectionMatrix), With<Camera>>,
		Query<&mut Uniforms>,
	)>,
) {
	let (camera_transform, view_c, projection_c) = q.q0().single().unwrap();
	let (view, projection) = (view_c.0, projection_c.0);
	let camera_position = camera_transform.translation;
	for mut uniforms in q.q1_mut().iter_mut() {
		uniforms.view_pos = camera_position;
		uniforms.view = view;
		uniforms.projection = projection;
		uniforms.light_pos = camera_position;
	}
}

#[repr(C)]
struct Uniforms {
	model: Mat4,
	view: Mat4,
	projection: Mat4,
	view_pos: Vec3,
	light_pos: Vec3,
	light_color: Vec3,
	object_color: Vec3,
}

impl Default for Uniforms {
	fn default() -> Self {
		Self {
			model: Mat4::IDENTITY,
			view: Mat4::IDENTITY,
			projection: Mat4::IDENTITY,
			view_pos: Vec3::ZERO,
			light_pos: Vec3::ZERO,
			light_color: vec3(1.0, 1.0, 1.0),
			object_color: vec3(1.0, 1.0, 1.0),
		}
	}
}

mod shader {
	use miniquad::UniformType;

	pub const VERTEX: &str = r#"#version 330 core
	in vec3 pos;
	in vec3 normal;
	in vec2 uv;

	out vec3 FragPos;
	out vec3 Normal;

	uniform mat4 model;
	uniform mat4 view;
	uniform mat4 projection;

	void main() {
		FragPos = vec3(model * vec4(pos, 1.0));
		Normal = mat3(transpose(inverse(model))) * normal;

		gl_Position = projection * view * vec4(FragPos, 1.0);
	}
	"#;

	pub const FRAGMENT: &str = r#"#version 330 core
	out vec4 FragColor;

	in vec3 Normal;
	in vec3 FragPos;

	uniform vec3 light_pos;
	uniform vec3 view_pos;
	uniform vec3 light_color;
	uniform vec3 object_color;

	vec3 ambient_color = vec3(1.0) * 0.3;

	void main() {
		// diffuse
		vec3 norm = normalize(Normal);
		vec3 light_dir = normalize(light_pos - FragPos);
		float diff = max(dot(norm, light_dir), 0.0);
		vec3 diffuse = diff * light_color;

		// specular
		float specular_strength = 0.5;
		vec3 view_dir = normalize(view_pos - FragPos);
		vec3 reflect_dir = reflect(-light_dir, norm);
		float spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32);
		vec3 specular = specular_strength * spec * light_color;

		vec3 result = (ambient_color + diffuse + specular) * object_color;
		FragColor = vec4(result, 1.0);
	}
	"#;

	pub const TEXTURES: [&str; 0] = [];
	pub const UNIFORMS: [(&str, UniformType); 7] = [
		("model", UniformType::Mat4),
		("view", UniformType::Mat4),
		("projection", UniformType::Mat4),
		("light_pos", UniformType::Float3),
		("view_pos", UniformType::Float3),
		("light_color", UniformType::Float3),
		("object_color", UniformType::Float3),
	];
}
