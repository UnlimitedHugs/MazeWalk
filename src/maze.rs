use std::cmp::Ordering;

use super::{
	maze_gen::{self, GridDirection, GridMaze},
	rendering::*,
	utils::Color,
	utils::Cube,
};
use bevy::{
	input::mouse::MouseMotion,
	math::{vec2, vec3},
	prelude::*,
};
use miniquad::{Comparison, CullFace, PipelineParams};
use rand::{seq::IteratorRandom, seq::SliceRandom, thread_rng, Rng};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemLabel)]
enum SystemLabels {
	CameraLookInput,
	PlayerMovement,
}

pub struct MazePlugin;
impl Plugin for MazePlugin {
	fn build(&self, app: &mut AppBuilder) {
		use SystemLabels::*;
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
		.add_event::<ChunkEntered>()
		.add_event::<ChunkExited>()
		.add_startup_system(spawn_initial_chunk.system())
		.add_system(camera_look_input.system().label(CameraLookInput))
		.add_system(expand_euler_rotation.system().after(CameraLookInput))
		.add_system(player_movement.system().label(PlayerMovement))
		.add_system(collide_with_walls.system().after(PlayerMovement))
		.add_system(track_current_chunk.system().after(PlayerMovement))
		.add_system(spawn_additional_chunk.system())
		.add_system(despawn_traversed_chunks.system())
		.add_system(update_hover_mode.system())
		.add_system_to_stage(RenderStage::PreRender, update_uniforms.system());
	}
}

const PI: f32 = std::f32::consts::PI;
const CELL_SIZE: f32 = 1.0;
const CHUNK_SIZE: i32 = 17;

struct MazeAssets {
	cube_mesh: Handle<Mesh>,
	shader: Handle<Shader>,
	wall_colors: Vec<Color>,
}

fn spawn_initial_chunk(
	mut cmd: Commands,
	mut meshes: ResMut<Assets<Mesh>>,
	mut shaders: ResMut<Assets<Shader>>,
) {
	let maze_assets = {
		let mut rng = thread_rng();
		let cube_mesh = meshes.add(Cube::new(CELL_SIZE).into());
		let shader = shaders.add(Shader::new(
			shader::VERTEX,
			shader::FRAGMENT,
			&shader::TEXTURES,
			&shader::UNIFORMS,
		));

		let wall_colors = {
			let num_samples = 8;
			let hue_offset = rng.gen_range(0.0..360.0);
			let mut colors = (0..num_samples)
				.map(|i| {
					Color::hsl(
						(((360 / num_samples) * i) as f32 + hue_offset) % 360.,
						0.4,
						0.8,
					)
				})
				.collect::<Vec<_>>();
			colors.shuffle(&mut rng);
			colors
		};
		MazeAssets {
			cube_mesh,
			shader,
			wall_colors,
		}
	};

	let first_chunk = generate_chunk(&mut cmd, &maze_assets, 0, ChunkCoords::ZERO, None);
	cmd.insert_resource(maze_assets);

	let camera_transform = {
		let (entrance_x, entrance_z) =
			maze_to_grid(first_chunk.maze.idx_to_pos(first_chunk.entrance.node));
		let random_entrance_neighbor = first_chunk
			.maze
			.get_links(&first_chunk.maze[first_chunk.entrance.node])
			.into_iter()
			.choose(&mut thread_rng())
			.expect("entrance neighbor");
		let (neighbor_x, neighbor_z) =
			maze_to_grid(first_chunk.maze.idx_to_pos(random_entrance_neighbor.pos()));
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

struct Wall;
#[derive(Clone)]
struct SidedNode {
	node: usize,
	side: GridDirection,
}
#[derive(Clone)]
struct Chunk {
	index: usize,
	coords: ChunkCoords,
	maze: GridMaze,
	entrance: SidedNode,
	exit: SidedNode,
}
#[derive(Clone, Copy)]
struct ChunkCoords(IVec2);
impl ChunkCoords {
	const ZERO: ChunkCoords = ChunkCoords(IVec2::ZERO);
	fn as_rect(self) -> Rect<f32> {
		let (x, y) = (
			(self.0.x * CHUNK_SIZE) as f32,
			(self.0.y * CHUNK_SIZE) as f32,
		);
		Rect {
			left: x,
			right: x + CHUNK_SIZE as f32,
			top: y,
			bottom: y + CHUNK_SIZE as f32,
		}
	}
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

struct ChunkEntered(Entity);
struct ChunkExited(Entity);

fn track_current_chunk(
	q_chunks: Query<(Entity, &Chunk)>,
	q_cam: Query<&GlobalTransform, With<Camera>>,
	mut entered_event: EventWriter<ChunkEntered>,
	mut exited_event: EventWriter<ChunkExited>,
	mut current_chunk: Local<Option<Entity>>,
) {
	let cam_pos = q_cam.single().unwrap().translation;
	let contains_camera = q_chunks
		.iter()
		.find(|(_, c)| c.coords.as_rect().contains(vec2(cam_pos.x, cam_pos.z)));
	if let Some((cam_chunk_ent, _)) = contains_camera {
		if *current_chunk != Some(cam_chunk_ent) {
			if let Some(exited) = *current_chunk {
				exited_event.send(ChunkExited(exited));
			}
			*current_chunk = Some(cam_chunk_ent);
			entered_event.send(ChunkEntered(cam_chunk_ent));
		}
	}
}

fn spawn_additional_chunk(
	mut cmd: Commands,
	assets: Res<MazeAssets>,
	q: Query<(Entity, &Chunk)>,
	mut entered_event: EventReader<ChunkEntered>,
) {
	let (last_chunk_ent, last_chunk_data) = q
		.iter()
		.max_by_key(|(_, c)| c.index)
		.expect("get last chunk");
	let entered_last_chunk = entered_event
		.iter()
		.any(|ChunkEntered(e)| *e == last_chunk_ent);
	if entered_last_chunk {
		let (next_chunk_coords, next_chunk_entrance) = {
			let base_chunk = last_chunk_data;
			let next_chunk_dir: IVec2 = base_chunk.exit.side.get_offset().into();
			let exit_pos: IVec2 = base_chunk.maze.idx_to_pos(base_chunk.exit.node).into();
			let next_chunk_coords = base_chunk.coords.0 + next_chunk_dir;
			let maze_size = base_chunk.maze.dimensions().0 as i32;
			let entrance_pos = (base_chunk.coords.0 * maze_size + exit_pos + next_chunk_dir)
				- next_chunk_coords * maze_size;
			debug_assert!(
				entrance_pos.x >= 0
					&& entrance_pos.y >= 0
					&& entrance_pos.x < maze_size
					&& entrance_pos.y < maze_size
			);
			let entrance_index = GridMaze::idx_1d(
				entrance_pos.y as usize,
				entrance_pos.x as usize,
				maze_size as usize,
			);
			(
				ChunkCoords(next_chunk_coords),
				SidedNode {
					node: entrance_index,
					side: base_chunk.exit.side.opposite(),
				},
			)
		};

		generate_chunk(
			&mut cmd,
			&assets,
			last_chunk_data.index + 1,
			next_chunk_coords,
			Some(next_chunk_entrance),
		);
	}
}

fn despawn_traversed_chunks(
	mut cmd: Commands,
	q: Query<(Entity, &Chunk)>,
	mut exited_event: EventReader<ChunkExited>,
) {
	for ChunkExited(exited_ent) in exited_event.iter() {
		let exited_index = q.get(*exited_ent).expect("resolve exited chunk").1.index;
		for (ent, chunk) in q.iter() {
			if chunk.index < exited_index {
				cmd.entity(ent).despawn_recursive();
			}
		}
	}
}

fn generate_chunk(
	cmd: &mut Commands,
	assets: &MazeAssets,
	index: usize,
	coords: ChunkCoords,
	known_entrance: Option<SidedNode>,
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

	let make_entrance_passage = known_entrance.is_some();
	let (entrance, exit) = {
		let entrance = known_entrance.unwrap_or_else(|| {
			let side = GridDirection::ALL[rng.gen_range(0..4)];
			SidedNode {
				node: maze
					.get_edge_nodes(side)
					.choose(&mut rng)
					.expect("select entrance node")
					.pos(),
				side,
			}
		});
		let distances = maze.distances(&maze[entrance.node]);
		let exit_pair = GridDirection::ALL
			.iter()
			.filter(|d| **d != entrance.side)
			.flat_map(|d| {
				maze.get_edge_nodes(*d)
					.iter()
					.map(|n| (n.pos(), *d))
					.collect::<Vec<_>>()
			})
			.max_by_key(|p| distances.get(&maze[p.0]))
			.expect("select exit node");
		(
			entrance,
			SidedNode {
				node: exit_pair.0,
				side: exit_pair.1,
			},
		)
	};

	{
		let mut make_outer_wall_passage = |n: &SidedNode| {
			let (x, z) = maze_to_grid(maze.idx_to_pos(n.node));
			let (x_off, z_off) = n.side.get_offset();
			grid[(z + z_off) as usize][(x + x_off) as usize] = false;
		};
		make_outer_wall_passage(&exit);
		if make_entrance_passage {
			make_outer_wall_passage(&entrance);
		}
	}

	let has_block = |x: i32, z: i32| {
		x >= 0 && x < CHUNK_SIZE && z >= 0 && z < CHUNK_SIZE && grid[z as usize][x as usize]
	};

	let chunk = Chunk {
		index,
		coords,
		maze,
		entrance,
		exit,
	};
	let chunk_entity = cmd.spawn_bundle((chunk.clone(),)).id();
	let wall_color: Vec3 = assets.wall_colors[index % assets.wall_colors.len()].into();

	for x in 0..CHUNK_SIZE {
		for z in 0..CHUNK_SIZE {
			if !has_block(x, z) {
				continue;
			}
			let transform =
				GlobalTransform::from_translation(vec3(x as f32, 0., z as f32) + coords.into());
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
				assets.cube_mesh.clone(),
				assets.shader.clone(),
				Uniforms {
					model: transform.compute_matrix(),
					object_color: wall_color,
					..Default::default()
				},
				edges,
				Parent(chunk_entity),
			));
		}
	}
	chunk
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
	fn contains(self, v: Vec2) -> bool;
}
impl RectExtension for Rect<f32> {
	fn intersects(self, other: Self) -> bool {
		!(other.right < self.left
			|| self.right < other.left
			|| other.top < self.bottom
			|| self.top < other.bottom)
	}
	fn contains(self, v: Vec2) -> bool {
		!(v.x < self.left || self.right < v.x || v.y < self.top || self.bottom < v.y)
	}
}

impl From<ChunkCoords> for Vec3 {
	fn from(o: ChunkCoords) -> Self {
		vec3((o.0.x * CHUNK_SIZE) as f32, 0., (o.0.y * CHUNK_SIZE) as f32)
	}
}

fn maze_to_grid((x, z): (i32, i32)) -> (i32, i32) {
	(x * 2 + 1, z * 2 + 1)
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn rect_contains() {
		let r = Rect {
			left: 1.,
			right: 2.,
			top: 3.,
			bottom: 4.,
		};
		assert_eq!(r.contains(vec2(0.5, 0.5)), false);
		assert_eq!(r.contains(vec2(1.5, 0.5)), false);
		assert_eq!(r.contains(vec2(1.5, 3.5)), true);
		assert_eq!(r.contains(vec2(3.5, 3.5)), false);
		assert_eq!(r.contains(vec2(2.5, 4.5)), false);
	}
}
