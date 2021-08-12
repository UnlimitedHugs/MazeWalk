use glam::Vec2;

#[derive(Clone, Copy)]
pub struct Rect {
	pub left: f32,
	pub right: f32,
	pub top: f32,
	pub bottom: f32,
}

impl Rect {
	pub fn intersects(self, other: Self) -> bool {
		!(other.right < self.left
			|| self.right < other.left
			|| other.top < self.bottom
			|| self.top < other.bottom)
	}

	pub fn contains(self, v: Vec2) -> bool {
		!(v.x < self.left || self.right < v.x || v.y < self.top || self.bottom < v.y)
	}
}

#[cfg(test)]
mod tests {
	use glam::vec2;

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
