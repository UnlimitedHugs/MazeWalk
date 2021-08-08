use crate::prelude::*;
use crate::utils::GlobalTransform;
use glam::{Mat4, Vec3};
use std::ops::Range;

pub fn plugin(app: &mut AppBuilder) {
	app.add_system_to_stage(CoreStage::PostUpdate, update_view_matrix.system())
		.add_system_to_stage(CoreStage::PostUpdate, update_projection_matrix.system());
}

#[derive(Bundle)]
pub struct CameraBundle {
	pub camera: Camera,
	pub transform: GlobalTransform,
	pub view: ViewMatrix,
	pub projection: ProjectionMatrix,
}

impl Default for CameraBundle {
	fn default() -> Self {
		Self {
			camera: Default::default(),
			transform: GlobalTransform::looking_at(GlobalTransform::identity(), -Vec3::Z, Vec3::Y),
			view: Default::default(),
			projection: Default::default(),
		}
	}
}

pub struct Camera {
	pub field_of_view: f32,
	pub clipping_distance: Range<f32>,
}
impl Default for Camera {
	fn default() -> Self {
		Camera {
			field_of_view: 60.0,
			clipping_distance: 0.01..100.0,
		}
	}
}

#[derive(Default)]
pub struct ViewMatrix(pub Mat4);

#[derive(Default)]
pub struct ProjectionMatrix(pub Mat4);

fn update_projection_matrix(
	mut queries: QuerySet<(
		Query<(Entity, &Camera, &mut ProjectionMatrix)>,
		Query<Entity, Changed<Camera>>,
	)>,
	mut resize_event: EventReader<WindowResize>,
	window: Res<WindowSize>,
) {
	let window_resized = resize_event.iter().count() > 0;
	let changed_cameras: Vec<_> = queries.q1().iter().collect();
	for (entity, cam, mut projection) in queries.q0_mut().iter_mut() {
		if changed_cameras.contains(&entity) || window_resized {
			projection.0 = Mat4::perspective_rh_gl(
				cam.field_of_view.to_radians(),
				window.width / window.height,
				cam.clipping_distance.start,
				cam.clipping_distance.end,
			)
		}
	}
}

fn update_view_matrix(
	mut query: Query<(&GlobalTransform, &mut ViewMatrix), Changed<GlobalTransform>>,
) {
	for (tx, mut view) in query.iter_mut() {
		view.0 = tx.compute_matrix().inverse();
	}
}
