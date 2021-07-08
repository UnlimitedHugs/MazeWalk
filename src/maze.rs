use std::cmp::Ordering;

use super::{
	maze_gen::{self, GridDirection, GridMaze, GridNode},
	rendering::*,
	utils::Cube,
};
use bevy::{input::mouse::MouseMotion, math::vec3, prelude::*};
use miniquad::{Comparison, CullFace, PipelineParams};
use rand::{seq::IteratorRandom, seq::SliceRandom, thread_rng, Rng};

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
		.add_system(player_movement.system().chain(collide_with_walls.system()))
		.add_system(update_hover_mode.system());
	}
}

const PI: f32 = std::f32::consts::PI;
const CELL_SIZE: f32 = 1.0;
const CHUNK_SIZE: i32 = 17;

struct Wall;

fn build_maze(
	mut cmd: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut shaders: ResMut<Assets<Shader>>,
) {
	let mut rng = thread_rng();
	let cube_mesh = meshes.add(Cube::new(CELL_SIZE).into());
	let shader = shaders.add(Shader::new(
		shader::VERTEX,
		shader::FRAGMENT,
		&shader::TEXTURES,
		&shader::UNIFORMS,
	));

	let first_chunk = generate_chunk(&mut cmd, cube_mesh, shader, false);
	let camera_transform = {
		let (entrance_z, entrance_x) =
			maze_to_grid(first_chunk.maze.idx_to_pos(first_chunk.entrance.node.pos()));
		let random_neighbor = first_chunk
			.maze
			.get_links(&first_chunk.entrance.node)
			.into_iter()
			.choose(&mut rng)
			.expect("entrance neighbor");
		let (neighbor_z, neighbor_x) =
			maze_to_grid(first_chunk.maze.idx_to_pos(random_neighbor.pos()));
		GlobalTransform::from_translation(vec3(entrance_x as f32, 0., entrance_z as f32))
			.looking_at(vec3(neighbor_x as f32, 0., neighbor_z as f32), Vec3::Y)
	};

	cmd.spawn_bundle(CameraBundle {
		transform: camera_transform,
		camera: Camera {
			field_of_view: 75.0,
			clipping_distance: 0.1..100.,
		},
		..Default::default()
	})
	.insert(RotationEuler {
		yaw: camera_transform.rotation.to_axis_angle().1,
		pitch: 0.,
	});
}

#[derive(Clone)]
struct SidedNode {
	node: GridNode,
	side: GridDirection,
}
#[derive(Clone)]
struct Chunk {
	maze: GridMaze,
	entrance: SidedNode,
	exit: SidedNode,
}

fn generate_chunk(
	cmd: &mut Commands,
	mesh: Handle<Mesh>,
	shader: Handle<Shader>,
	make_entrance: bool,
) -> Chunk {
	let mut rng = thread_rng();
	const MAZE_SIZE: usize = (CHUNK_SIZE as usize - 1) / 2;
	let maze = maze_gen::generate(MAZE_SIZE, MAZE_SIZE);
	let mut grid = {
		let mut grid = [[true; CHUNK_SIZE as usize]; CHUNK_SIZE as usize];
		for (maze_z, row) in maze.iter_rows().enumerate() {
			for (maze_x, node) in row.iter().enumerate() {
				let (z, x) = (maze_z * 2 + 1, maze_x * 2 + 1);
				grid[z][x] = false;
				if maze.has_link(node, GridDirection::Right) {
					grid[z][x + 1] = false;
				}
				if maze.has_link(node, GridDirection::Down) {
					grid[z + 1][x] = false;
				}
			}
		}
		grid
	};

	let (entrance, exit) = {
		let entrance_side = GridDirection::ALL[rng.gen_range(0..4)];
		let entrance_node = *maze
			.get_edge_nodes(entrance_side)
			.choose(&mut rng)
			.expect("select entrance node");
		let distances = maze.distances(&entrance_node);
		let exit_pair = GridDirection::ALL
			.iter()
			.filter(|d| **d != entrance_side)
			.flat_map(|d| {
				maze.get_edge_nodes(*d)
					.iter()
					.map(|n| (*n, *d))
					.collect::<Vec<_>>()
			})
			.max_by_key(|p| distances.get(&p.0))
			.expect("select exit node");
		(
			SidedNode {
				node: entrance_node,
				side: entrance_side,
			},
			SidedNode {
				node: exit_pair.0,
				side: exit_pair.1,
			},
		)
	};

	{
		let mut make_outer_wall_passage = |n: &SidedNode| {
			let (x, z) = maze_to_grid(maze.idx_to_pos(n.node.pos()));
			let (x_off, z_off) = n.side.get_offset();
			grid[(z + z_off) as usize][(x + x_off) as usize] = false;
		};
		make_outer_wall_passage(&exit);
		if make_entrance {
			make_outer_wall_passage(&entrance);
		}
	}

	let has_block = |x: i32, z: i32| {
		x >= 0 && x < CHUNK_SIZE && z >= 0 && z < CHUNK_SIZE && grid[x as usize][z as usize]
	};

	let chunk = Chunk {
		maze,
		entrance,
		exit,
	};
	let chunk_entity = cmd
		.spawn_bundle((
			Transform::default(),
			GlobalTransform::default(),
			chunk.clone(),
		))
		.id();

	for x in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			if !has_block(x, z) {
				continue;
			}
			let transform = Transform::from_translation(vec3(x as f32, 0., z as f32));
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
				GlobalTransform::default(),
				mesh.clone(),
				shader.clone(),
				Uniforms {
					model: transform.compute_matrix(),
					..Default::default()
				},
				edges,
				Parent(chunk_entity),
			));
		}
	}
	chunk
}

#[derive(Default)]
struct RotationEuler {
	yaw: f32,
	pitch: f32,
}

fn camera_look_input(
	mut q: Query<&mut RotationEuler, With<Camera>>,
	mut mouse_motion: EventReader<MouseMotion>,
) {
	let mut euler = q.single_mut().unwrap();
	let mouse_sensitivity = 0.006f32;
	let pitch_limit = 90.0f32.to_radians() * 0.99;
	for MouseMotion { delta } in mouse_motion.iter() {
		euler.yaw -= delta.x * mouse_sensitivity;
		euler.pitch = (euler.pitch - delta.y * mouse_sensitivity).clamp(-pitch_limit, pitch_limit);
	}
}

fn expand_euler_rotation(
	mut q: Query<(&mut GlobalTransform, &RotationEuler), Changed<RotationEuler>>,
) {
	// separate system to handle startup value
	for (mut tx, RotationEuler { yaw, pitch }) in q.iter_mut() {
		tx.rotation = Quat::from_rotation_ypr(*yaw, *pitch, 0.);
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
		let view_relative = Quat::from_rotation_y(euler.yaw) * (movement * 3. * t.delta_seconds());
		transform.translation += view_relative;
	}
}

fn collide_with_walls(
	mut q: QuerySet<(
		Query<(&GlobalTransform, &CollisionEdges), With<Wall>>,
		Query<(&mut GlobalTransform, Option<&NoClip>), With<Camera>>,
	)>,
) {
	let (cam_transform, noclip) = q.q1_mut().single_mut().unwrap();
	if noclip.is_some() {
		return;
	}
	let mut player_pos = cam_transform.translation;
	let player_size = 0.2f32;
	let wall_size = CELL_SIZE / 2.0;
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
		q.q1_mut().single_mut().unwrap().0.translation = player_pos;
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

fn update_hover_mode(
	mut cmd: Commands,
	mut q: Query<(Entity, &mut GlobalTransform, Option<&NoClip>), With<Camera>>,
	input: Res<Input<KeyCode>>,
) {
	let (cam_entity, mut cam_transform, cam_noclip) = q.single_mut().unwrap();
	if input.just_pressed(KeyCode::Space) {
		if cam_noclip.is_some() {
			cmd.entity(cam_entity).remove::<NoClip>();
			cam_transform.translation.y = 0.;
		} else {
			cmd.entity(cam_entity).insert(NoClip);
			cam_transform.translation.y = 4.;
		}
	}
}

struct NoClip;

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

fn maze_to_grid((z, x): (i32, i32)) -> (i32, i32) {
	(z * 2 + 1, x * 2 + 1)
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
