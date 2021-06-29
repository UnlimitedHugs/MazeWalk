use super::{rendering::*, utils::{Color, Cube}};
use bevy::{
	input::mouse::{MouseMotion, MouseWheel},
	math::vec3,
	prelude::*,
};
use miniquad::{Comparison, PipelineParams};
use rand::{Rng, RngCore};

type Window = bevy_miniquad::Window;

pub struct CubesDemoPlugin;

#[derive(Clone, Hash, Debug, PartialEq, Eq, SystemLabel)]
struct CameraInputSystem;

impl Plugin for CubesDemoPlugin {
	fn build(&self, app: &mut AppBuilder) {
		app.register_shader_uniforms::<CubeUniforms>()
			.insert_resource(RenderSettings {
				pipeline: PipelineParams {
					depth_test: Comparison::LessOrEqual,
					depth_write: true,
					..Default::default()
				},
				..Default::default()
			})
			.init_resource::<CameraState>()
			.add_startup_system(spawn_cubes.system())
			.add_system(
				handle_camera_input
					.system()
					.chain(update_camera_position.system()),
			)
			.add_system(update_cube_positions.system())
			.add_system(update_spinning.system())
			.add_system_to_stage(RenderStage::PreRender, update_cube_uniforms.system());
	}
}

const PI: f32 = std::f32::consts::PI;
const MIN_DISTANCE: f32 = 3.0;
const MAX_DISTANCE: f32 = 10.0;

struct CoreCube;

struct OrbitingCube {
	orbit_angle: Quat,
	orbit_distance: f32,
	orbit_eccentricity: Quat,
}

struct Spin {
	velocity: Quat,
}

fn spawn_cubes(
	mut cmd: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut shaders: ResMut<Assets<Shader>>,
) {
	let mesh = meshes.add(Cube::new(1.0).into());
	let orbiting_shader = shaders.add(Shader::new(
		shader::VERTEX,
		orbiting_shader::FRAGMENT,
		&shader::TEXTURES,
		&shader::UNIFORMS,
	));
	let core_shader = shaders.add(Shader::new(
		shader::VERTEX,
		core_shader::FRAGMENT,
		&shader::TEXTURES,
		&shader::UNIFORMS,
	));
	let mut rng = rand::thread_rng();

	for _ in 0..100 {
		let distance_multiplier: f32 = rng.gen();
		let cube = OrbitingCube {
			orbit_angle: Quat::from_rotation_y(rng.gen_angle()),
			orbit_distance: MIN_DISTANCE + distance_multiplier * (MAX_DISTANCE - MIN_DISTANCE),
			orbit_eccentricity: Quat::from_rotation_x(
				(-10.0 + rng.gen::<f32>() * 20.0).to_radians(),
			) * Quat::from_rotation_y(rng.gen_angle()),
		};
		let global_transform = GlobalTransform {
			translation: Vec3::ZERO,
			rotation: Quat::from_axis_angle(rng.gen_unit_vec3(), rng.gen_angle()),
			scale: Vec3::splat(0.2 + distance_multiplier * 0.3 + rng.gen::<f32>() * 0.2),
		};
		cmd.spawn_bundle((
			cube,
			Spin {
				velocity: Quat::from_axis_angle(rng.gen_unit_vec3(), rng.gen_angle()),
			},
			global_transform,
			mesh.clone(),
			orbiting_shader.clone(),
			CubeUniforms {
				object_color: Color::hsl(rng.gen::<f32>()*360., 0.5, 0.5).into(),
				..Default::default()
			},
		));
	}

	let core_transform = GlobalTransform {
		translation: Vec3::ZERO,
		rotation: Quat::from_rotation_x((10.0 + rng.gen::<f32>() * 20.0).to_radians())
			* Quat::from_rotation_z((10.0 + rng.gen::<f32>() * 20.0).to_radians())
			* Quat::from_rotation_y(rng.gen_angle()),
		scale: Vec3::splat(2.0),
	};
	cmd.spawn_bundle((
		CoreCube,
		Spin {
			velocity: Quat::from_rotation_y(15f32.to_radians()),
		},
		core_transform,
		mesh.clone(),
		core_shader,
		CubeUniforms::default(),
	));

	cmd.spawn_bundle(CameraBundle::default());
}

fn update_cube_positions(mut q: Query<(&mut GlobalTransform, &mut OrbitingCube)>, time: Res<Time>) {
	let base_velocity = 15.0f32.to_radians();
	for (mut transform, mut cube) in q.iter_mut() {
		let rotation_increment =
			(base_velocity / (cube.orbit_distance / MAX_DISTANCE)) * time.delta_seconds();
		cube.orbit_angle *= Quat::from_rotation_y(rotation_increment);
		transform.translation =
			cube.orbit_eccentricity * cube.orbit_angle * vec3(cube.orbit_distance, 0., 0.);
	}
}

fn update_spinning(mut q: Query<(&mut GlobalTransform, &Spin)>, time: Res<Time>) {
	for (mut transform, spin) in q.iter_mut() {
		let delta_time_adjusted_spin = Quat::IDENTITY.lerp(spin.velocity, time.delta_seconds());
		transform.rotation = delta_time_adjusted_spin * transform.rotation;
	}
}

struct CameraState {
	yaw: f32,
	pitch: f32,
	distance: f32,
}

impl Default for CameraState {
	fn default() -> Self {
		Self {
			yaw: 0.,
			pitch: -36.0f32.to_radians(),
			distance: 18.0,
		}
	}
}

fn handle_camera_input(
	mut state: ResMut<CameraState>,
	mut motion_event: EventReader<MouseMotion>,
	mut wheel_event: EventReader<MouseWheel>,
	window: Res<Window>,
) {
	for MouseMotion { delta } in motion_event.iter() {
		let Window { width, height, .. } = *window;
		state.yaw -= (delta.x / width as f32) * 10.;
		let pitch_limit = 90.0f32.to_radians() * 0.99;
		state.pitch =
			(state.pitch - (delta.y / height as f32) * 10.).clamp(-pitch_limit, pitch_limit);
	}
	for MouseWheel { y, .. } in wheel_event.iter() {
		state.distance += 1.0 * if *y > 0. { -1. } else { 1. };
	}
}

fn update_camera_position(
	state: Res<CameraState>,
	mut q: Query<&mut GlobalTransform, With<Camera>>,
) {
	if state.is_changed() {
		if let Ok(mut transform) = q.single_mut() {
			let camera_rotation =
				Mat4::from_rotation_y(state.yaw) * Mat4::from_rotation_x(state.pitch);
			transform.translation = camera_rotation.transform_point3(vec3(0.0, 0.0, state.distance));
			transform.look_at(Vec3::ZERO, Vec3::Y);
		}
	}
}

fn update_cube_uniforms(
	mut qs: QuerySet<(
		Query<(&mut CubeUniforms, &GlobalTransform)>,
		Query<(&ViewMatrix, &ProjectionMatrix, &GlobalTransform), With<Camera>>,
	)>,
) {
	let (view, projection, camera_tx) = qs.q1().single().unwrap();
	let (view, projection, view_pos) =
		(view.0.clone(), projection.0.clone(), camera_tx.translation);
	for (mut uniforms, transform) in qs.q0_mut().iter_mut() {
		uniforms.model = transform.compute_matrix();
		uniforms.view = view;
		uniforms.projection = projection;
		uniforms.view_pos = view_pos;
	}
}

#[repr(C)]
struct CubeUniforms {
	model: Mat4,
	view: Mat4,
	projection: Mat4,
	light_pos: Vec3,
	view_pos: Vec3,
	light_color: Vec3,
	object_color: Vec3,
}

impl Default for CubeUniforms {
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

mod orbiting_shader {
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
}

mod core_shader {
	pub const FRAGMENT: &str = r#"#version 330 core
	out vec4 FragColor;

	uniform vec3 object_color;

	void main() {
		FragColor = vec4(object_color, 1.0);
	}
	"#;
}

trait RngExtensions: RngCore {
	fn gen_angle(&mut self) -> f32 {
		self.gen_range(0.0..PI * 2.)
	}
	fn gen_unit_vec3(&mut self) -> Vec3 {
		let mut f = || self.gen::<f32>() - 0.5;
		vec3(f(), f(), f()).normalize()
	}
}
impl<R: RngCore> RngExtensions for R {}
