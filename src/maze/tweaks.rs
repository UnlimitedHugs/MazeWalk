use super::Material;

pub struct Tweaks {
	pub ambient_light_intensity: f32,
	pub ceiling_material: Material,
	pub wall_material: Material,
	pub floor_material: Material,
	pub mouse_sensitivity: f32,
	pub mouse_delta_cap: f32,
}
impl Default for Tweaks {
	fn default() -> Self {
		Self {
			ambient_light_intensity: 0.1,
			ceiling_material: Material {
				color: 0xFFFFFF,
				normal_intensity: 0.6,
				specular_strength: 0.1,
				shininess: 8.0,
			},
			wall_material: Material {
				color: 0xFFFFFF,
				normal_intensity: 0.2,
				specular_strength: 0.2,
				shininess: 64.0,
			},
			floor_material: Material {
				color: 0xFFFFFF,
				normal_intensity: 1.0,
				specular_strength: 0.2,
				shininess: 32.0,
			},
			mouse_sensitivity: 0.0045,
			mouse_delta_cap: 60.,
		}
	}
}