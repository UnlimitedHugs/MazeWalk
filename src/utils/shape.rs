#![allow(dead_code)]
// adapted from bevy_render/src/mesh/shape

use crate::rendering::{Mesh, Vertex};
use glam::{vec2, vec3, Vec2, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct Cube {
	pub size: f32,
}

impl Cube {
	pub fn new(size: f32) -> Cube {
		Cube { size }
	}
}

impl Default for Cube {
	fn default() -> Self {
		Cube { size: 1.0 }
	}
}

impl From<Cube> for Mesh {
	fn from(cube: Cube) -> Self {
		BoxShape::new(cube.size, cube.size, cube.size).into()
	}
}

#[derive(Debug, Copy, Clone)]
pub struct BoxShape {
	pub min_x: f32,
	pub max_x: f32,

	pub min_y: f32,
	pub max_y: f32,

	pub min_z: f32,
	pub max_z: f32,
}

impl BoxShape {
	pub fn new(x_length: f32, y_length: f32, z_length: f32) -> BoxShape {
		BoxShape {
			max_x: x_length / 2.0,
			min_x: -x_length / 2.0,
			max_y: y_length / 2.0,
			min_y: -y_length / 2.0,
			max_z: z_length / 2.0,
			min_z: -z_length / 2.0,
		}
	}
}

impl Default for BoxShape {
	fn default() -> Self {
		BoxShape::new(2.0, 1.0, 1.0)
	}
}

impl From<BoxShape> for Mesh {
	fn from(sp: BoxShape) -> Self {
		#[rustfmt::skip]
		let vertex_data = [
			// Top
			(vec3(sp.min_x, sp.min_y, sp.max_z), vec3(0., 0., 1.0), vec2(0., 0.)),
			(vec3(sp.max_x, sp.min_y, sp.max_z), vec3(0., 0., 1.0), vec2(1.0, 0.)),
			(vec3(sp.max_x, sp.max_y, sp.max_z), vec3(0., 0., 1.0), vec2(1.0, 1.0)),
			(vec3(sp.min_x, sp.max_y, sp.max_z), vec3(0., 0., 1.0), vec2(0., 1.0)),
			// Bottom
			(vec3(sp.min_x, sp.max_y, sp.min_z), vec3(0., 0., -1.0), vec2(1.0, 0.)),
			(vec3(sp.max_x, sp.max_y, sp.min_z), vec3(0., 0., -1.0), vec2(0., 0.)),
			(vec3(sp.max_x, sp.min_y, sp.min_z), vec3(0., 0., -1.0), vec2(0., 1.0)),
			(vec3(sp.min_x, sp.min_y, sp.min_z), vec3(0., 0., -1.0), vec2(1.0, 1.0)),
			// Right
			(vec3(sp.max_x, sp.min_y, sp.min_z), vec3(1.0, 0., 0.), vec2(0., 0.)),
			(vec3(sp.max_x, sp.max_y, sp.min_z), vec3(1.0, 0., 0.), vec2(1.0, 0.)),
			(vec3(sp.max_x, sp.max_y, sp.max_z), vec3(1.0, 0., 0.), vec2(1.0, 1.0)),
			(vec3(sp.max_x, sp.min_y, sp.max_z), vec3(1.0, 0., 0.), vec2(0., 1.0)),
			// Left
			(vec3(sp.min_x, sp.min_y, sp.max_z), vec3(-1.0, 0., 0.), vec2(1.0, 0.)),
			(vec3(sp.min_x, sp.max_y, sp.max_z), vec3(-1.0, 0., 0.), vec2(0., 0.)),
			(vec3(sp.min_x, sp.max_y, sp.min_z), vec3(-1.0, 0., 0.), vec2(0., 1.0)),
			(vec3(sp.min_x, sp.min_y, sp.min_z), vec3(-1.0, 0., 0.), vec2(1.0, 1.0)),
			// Front
			(vec3(sp.max_x, sp.max_y, sp.min_z), vec3(0., 1.0, 0.), vec2(1.0, 0.)),
			(vec3(sp.min_x, sp.max_y, sp.min_z), vec3(0., 1.0, 0.), vec2(0., 0.)),
			(vec3(sp.min_x, sp.max_y, sp.max_z), vec3(0., 1.0, 0.), vec2(0., 1.0)),
			(vec3(sp.max_x, sp.max_y, sp.max_z), vec3(0., 1.0, 0.), vec2(1.0, 1.0)),
			// Back
			(vec3(sp.max_x, sp.min_y, sp.max_z), vec3(0., -1.0, 0.), vec2(0., 0.)),
			(vec3(sp.min_x, sp.min_y, sp.max_z), vec3(0., -1.0, 0.), vec2(1.0, 0.)),
			(vec3(sp.min_x, sp.min_y, sp.min_z), vec3(0., -1.0, 0.), vec2(1.0, 1.0)),
			(vec3(sp.max_x, sp.min_y, sp.min_z), vec3(0., -1.0, 0.), vec2(0., 1.0)),
		];

		let indices: Vec<u16> = vec![
			0, 1, 2, 2, 3, 0, // top
			4, 5, 6, 6, 7, 4, // bottom
			8, 9, 10, 10, 11, 8, // right
			12, 13, 14, 14, 15, 12, // left
			16, 17, 18, 18, 19, 16, // front
			20, 21, 22, 22, 23, 20, // back
		];

		mesh_from_vertex_data(&vertex_data, indices, 24)
	}
}

/// A rectangle on the XY plane.
#[derive(Debug, Copy, Clone)]
pub struct Quad {
	/// Full width and height of the rectangle.
	pub size: Vec2,
	/// Flips the texture coords of the resulting vertices.
	pub flip: bool,
}

impl Default for Quad {
	fn default() -> Self {
		Quad::new(Vec2::ONE)
	}
}

impl Quad {
	pub fn new(size: Vec2) -> Self {
		Self { size, flip: false }
	}

	pub fn flipped(size: Vec2) -> Self {
		Self { size, flip: true }
	}
}

impl From<Quad> for Mesh {
	fn from(quad: Quad) -> Self {
		let extent_x = quad.size.x / 2.0;
		let extent_y = quad.size.y / 2.0;

		let north_west = vec2(-extent_x, extent_y);
		let north_east = vec2(extent_x, extent_y);
		let south_west = vec2(-extent_x, -extent_y);
		let south_east = vec2(extent_x, -extent_y);

		#[rustfmt::skip]
		let vertex_data = if quad.flip {
			[
				(vec3(south_east.x, south_east.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(1.0, 1.0)),
				(vec3(north_east.x, north_east.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(1.0, 0.0)),
				(vec3(north_west.x, north_west.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(0.0, 0.0)),
				(vec3(south_west.x, south_west.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
			]
		} else {
			[
				(vec3(south_west.x, south_west.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(0.0, 1.0)),
				(vec3(north_west.x, north_west.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(0.0, 0.0)),
				(vec3(north_east.x, north_east.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(1.0, 0.0)),
				(vec3(south_east.x, south_east.y, 0.0), vec3(0.0, 0.0, 1.0), vec2(1.0, 1.0)),
			]
		};
		let indices: Vec<u16> = vec![0, 2, 1, 0, 3, 2];

		mesh_from_vertex_data(&vertex_data, indices, 4)
	}
}

/// A square on the XZ plane.
#[derive(Debug, Copy, Clone)]
pub struct Plane {
	/// The total side length of the square.
	pub size: f32,
	pub tiling: f32,
}

impl Plane {
	pub fn new(size: f32, tiling: f32) -> Self {
		Self { size, tiling }
	}
}

impl Default for Plane {
	fn default() -> Self {
		Plane {
			size: 1.0,
			tiling: 1.0,
		}
	}
}

impl From<Plane> for Mesh {
	fn from(plane: Plane) -> Self {
		let Plane { size, tiling } = plane;
		let extent = size / 2.0;

		#[rustfmt::skip]
		let vertex_data = [
			(vec3(extent,  0.0, -extent), vec3(0.0, 1.0, 0.0), vec2(tiling, tiling)),
			(vec3(extent,  0.0,  extent), vec3(0.0, 1.0, 0.0), vec2(tiling, 0.0)),
			(vec3(-extent, 0.0,  extent), vec3(0.0, 1.0, 0.0), vec2(0.0, 0.0)),
			(vec3(-extent, 0.0, -extent), vec3(0.0, 1.0, 0.0), vec2(0.0, tiling)),
		];

		let indices: Vec<u16> = vec![0, 2, 1, 0, 3, 2];

		mesh_from_vertex_data(&vertex_data, indices, 4)
	}
}

fn mesh_from_vertex_data(
	vertex_data: &[(Vec3, Vec3, Vec2)],
	indices: Vec<u16>,
	capacity: usize,
) -> Mesh {
	let mut vertices = Vec::<_>::with_capacity(capacity);
	for (pos, normal, uv) in vertex_data.iter() {
		vertices.push(Vertex {
			pos: *pos,
			normal: *normal,
			uv: *uv,
		})
	}
	Mesh { vertices, indices }
}
