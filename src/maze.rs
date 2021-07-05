use std::cmp::Ordering;

use super::{rendering::*, utils::Cube, maze_gen::{self, LinkDirection}};
use bevy::{
	input::mouse::MouseMotion,
	math::{vec2, vec3},
	prelude::*,
};
use miniquad::{Comparison, CullFace, PipelineParams};
//use rand::Rng;

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
		.add_system(
			camera_look_input
				.system()
				.chain(expand_euler_rotation.system()),
		)
		.add_system(player_movement.system().chain(collide_with_walls.system()));
	}
}

const PI: f32 = std::f32::consts::PI;
const CELL_SIZE: f32 = 1.0;

struct Wall;

fn build_maze(
	mut cmd: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut shaders: ResMut<Assets<Shader>>,
) {
	let cube_mesh = meshes.add(Cube::new(CELL_SIZE).into());
	let shader = shaders.add(Shader::new(
		shader::VERTEX,
		shader::FRAGMENT,
		&shader::TEXTURES,
		&shader::UNIFORMS,
	));

	const GRID_SIZE: i32 = 17;
	let grid = {
		let mut grid = [[true; GRID_SIZE as usize]; GRID_SIZE as usize];
		const MAZE_SIZE: usize = GRID_SIZE as usize / 2;
		let maze = maze_gen::generate(MAZE_SIZE, MAZE_SIZE);
		for (maze_z, row) in maze.iter_rows().enumerate() {
			for (maze_x, node) in row.iter().enumerate() {
				let (z, x) = (maze_z * 2 + 1, maze_x * 2 + 1);
				grid[z][x] = false;
				if maze.has_link(node, LinkDirection::Right) {
					grid[z][x + 1] = false;
				}
				if maze.has_link(node, LinkDirection::Down) {
					grid[z + 1][x] = false;
				}
			}
		}
		grid
	};

	//let mut rng = rand::thread_rng();
	//const GRID_SIZE: i32 = 16;
	// let grid = {
	// 	const SIZE: usize = GRID_SIZE as usize;
	// 	let mut arr = [[false; SIZE]; SIZE];
	// 	for x in 0..SIZE {
	// 		for z in 0..SIZE {
	// 			arr[x][z] = rng.gen_ratio(1, 3);
	// 		}
	// 	}
	// 	arr
	// };

	// const GRID_SIZE: i32 = 8;
	// let grid = [ // H shapes
	// 	[false, false, false, false, false, false, false, false],
	// 	[false, true, true, true, false, true, false, true],
	// 	[false, false, true, false, false, true, true, true],
	// 	[false, true, true, true, false, true, false, true],
	// 	[false, false, false, false, false, false, false, false],
	// 	[false, false, false, false, false, false, false, false],
	// 	[false, false, false, false, false, false, false, false],
	// 	[false, false, false, false, false, false, false, false],
	// ];

	// const GRID_SIZE: i32 = 8;
	// let grid = [ // hor/vert segments
	// 	[false,false,false,false,false,false,false,false],
	// 	[false,true ,false,true ,false,false,false,false],
	// 	[false,false,false,true ,false,false,false,false],
	// 	[false,false,false,false,false,false,false,false],
	// 	[false,true, true, false,false,false,false,false],
	// 	[false,false,false,false,false,false,false,false],
	// 	[false,false,false,false,false,false,false,false],
	// 	[false,false,false,false,false,false,false,false],
	// ];

	let has_block = |x: i32, z: i32| {
		x >= 0 && x < GRID_SIZE && z >= 0 && z < GRID_SIZE && grid[x as usize][z as usize]
	};

	for x in 0..GRID_SIZE {
		for z in 0..GRID_SIZE {
			if !has_block(x, z) {
				continue;
			}
			let transform = GlobalTransform::from_translation(vec3(x as f32, 0., z as f32));
			let edges = CollisionEdges {
				edges: CollisionEdge::ALL
					.iter()
					.filter_map(|e| {
						let (dx, dz) = e.get_direction();
						if !has_block(x + dx, z + dz) {
							Some(e)
						} else {
							None
						}
					})
					.copied()
					.collect(),
			};
			cmd.spawn_bundle((
				Wall,
				transform,
				cube_mesh.clone(),
				shader.clone(),
				Uniforms {
					model: transform.compute_matrix(),
					..Default::default()
				},
				edges,
			));
		}
	}

	cmd.spawn_bundle(CameraBundle {
		transform: GlobalTransform::from_translation(vec3(1., 0., GRID_SIZE as f32 - 2.)),
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

fn camera_look_input(
	mut q: Query<&mut RotationEuler, With<Camera>>,
	mut mouse_motion: EventReader<MouseMotion>,
) {
	let mut euler = q.single_mut().unwrap();
	let mouse_sensitivity = 0.006f32;
	let pitch_limit = 90.0f32.to_radians() * 0.99;
	for MouseMotion { delta } in mouse_motion.iter() {
		euler.0 = vec2(
			euler.0.x - delta.x * mouse_sensitivity,
			(euler.0.y - delta.y * mouse_sensitivity).clamp(-pitch_limit, pitch_limit),
		);
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

fn player_movement(
	mut q: Query<(&mut GlobalTransform, &RotationEuler), With<Camera>>,
	key: Res<Input<KeyCode>>,
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

	let (mut transform, euler) = q.single_mut().unwrap();
	if movement != Vec3::ZERO {
		let view_relative = Quat::from_rotation_y(euler.0.x) * (movement * 3. * t.delta_seconds());
		transform.translation += view_relative;
	}
}

fn collide_with_walls(
	mut q: QuerySet<(
		Query<(&GlobalTransform, &CollisionEdges), With<Wall>>,
		Query<&mut GlobalTransform, With<Camera>>,
	)>,
) {
	let player_size = 0.2f32;
	let wall_size = CELL_SIZE / 2.0;
	let mut player_pos = q.q1_mut().single_mut().unwrap().translation;
	let mut position_adjusted = false;
	for (
		GlobalTransform {
			translation: wall_pos,
			..
		},
		edges,
	) in q.q0().iter()
	{
		let player_rect = Rect {
			left: player_pos.x - player_size,
			right: player_pos.x + player_size,
			top: player_pos.z + player_size,
			bottom: player_pos.z - player_size,
		};
		let wall_rect = Rect {
			left: wall_pos.x - wall_size,
			right: wall_pos.x + wall_size,
			top: wall_pos.z + wall_size,
			bottom: wall_pos.z - wall_size,
		};
		if player_rect.intersects(wall_rect) {
			if let Some(closest_edge) = edges.get_closest(*wall_pos, player_pos) {
				closest_edge.clip(*wall_pos, &mut player_pos, player_size);
				position_adjusted = true;
			}
		}
	}
	if position_adjusted {
		q.q1_mut().single_mut().unwrap().translation = player_pos;
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

#[derive(Clone, Copy, Debug)]
enum CollisionEdge {
	NegX,
	PosX,
	NegZ,
	PosZ,
}
impl CollisionEdge {
	const ALL: [CollisionEdge; 4] = [
		CollisionEdge::NegX,
		CollisionEdge::PosX,
		CollisionEdge::NegZ,
		CollisionEdge::PosZ,
	];
	fn get_angle(&self) -> f32 {
		// z.atan2(x).
		match self {
			CollisionEdge::NegX => 0.,
			CollisionEdge::PosX => PI,
			CollisionEdge::NegZ => PI / 2.,
			CollisionEdge::PosZ => -PI / 2.,
		}
	}
	fn get_direction(&self) -> (i32, i32) {
		match self {
			CollisionEdge::NegX => (-1, 0),
			CollisionEdge::PosX => (1, 0),
			CollisionEdge::NegZ => (0, -1),
			CollisionEdge::PosZ => (0, 1),
		}
	}
	fn get_offset(&self, collider_size: f32) -> Vec3 {
		let (dx, dz) = self.get_direction();
		vec3(dx as f32, 0., dz as f32) * ((CELL_SIZE / 2.) + collider_size)
	}
	fn clip(&self, parent_pos: Vec3, collider_pos: &mut Vec3, collider_size: f32) {
		let (self_x, _, self_z) = (parent_pos + self.get_offset(collider_size)).into();
		match self {
			CollisionEdge::NegX => collider_pos.x = collider_pos.x.min(self_x),
			CollisionEdge::PosX => collider_pos.x = collider_pos.x.max(self_x),
			CollisionEdge::NegZ => collider_pos.z = collider_pos.z.min(self_z),
			CollisionEdge::PosZ => collider_pos.z = collider_pos.z.max(self_z),
		}
	}
}

struct CollisionEdges {
	edges: Vec<CollisionEdge>,
}

impl CollisionEdges {
	fn get_closest(
		&self,
		parent_cell_pos: Vec3,
		colliding_body_pos: Vec3,
	) -> Option<CollisionEdge> {
		let body_dir = parent_cell_pos - colliding_body_pos;
		let angle_to_body = body_dir.z.atan2(body_dir.x);
		let get_angle_difference = |e: CollisionEdge| {
			let diff = e.get_angle() - angle_to_body;
			diff.sin().atan2(diff.cos()).abs()
		};
		self.edges
			.iter()
			.filter_map(|e| {
				let diff = get_angle_difference(*e);
				if diff < PI / 2. {
					Some((diff, e))
				} else {
					None
				}
			})
			.min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(Ordering::Equal))
			.map(|o| *o.1)
	}
}

trait RectExtension {
	fn intersects(self, other: Self) -> bool;
}
impl<T: Reflect + PartialEq + PartialOrd> RectExtension for Rect<T> {
	fn intersects(self, other: Self) -> bool {
		!(other.right < self.left
			|| self.right < other.left
			|| other.top < self.bottom
			|| self.top < other.bottom)
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
