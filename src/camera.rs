use nalgebra as na;

const FOV: f32 = 4.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
	pub width: f32,
	pub height: f32,
	pub projection: na::Perspective3<f32>,
	pub position: na::Point3<f32>,
	pub rotation: na::Vector3<f32>,
	pub scaling: na::Vector3<f32>,
}

impl Default for Camera {
	fn default() -> Self {
		Self {
			width: Default::default(),
			height: Default::default(),
			projection: na::Perspective3::new(1.0, 3.14 / FOV, 0.0001, 100.0),
			position: na::Point3::new(0.0, 0.0, -3.0),
			rotation: na::Vector3::new(0.0, 0.0, 0.0),
			scaling: na::Vector3::new(1.0, 1.0, 1.0),
		}
	}
}

impl Camera {
	pub fn new(width: f32, height: f32) -> Self {
		Self {
			width,
			height,
			projection: na::Perspective3::new(width / height, 3.14 / FOV, 0.0001, 100.0),
			..Default::default()
		}
	}

	pub fn width(&self) -> f32 {
		self.width
	}

	pub fn height(&self) -> f32 {
		self.height
	}

	pub fn view(&self) -> na::Matrix4<f32> {
		let mut mat4 = na::Matrix4::new_translation(&self.position.coords);
		mat4 *= self.scaling();
		mat4 *= self.rotation();
		mat4 *= na::Rotation3::face_towards(&na::Vector3::new(0.0, 0.0, -1.0), &na::Vector3::new(0.0, -1.0, 0.0))
			.to_homogeneous();

		mat4.try_inverse().unwrap()
	}

	pub fn projection(&self) -> na::Matrix4<f32> {
		self.projection.to_homogeneous()
	}

	pub fn rotation(&self) -> na::Matrix4<f32> {
		na::Matrix4::new_rotation(na::Vector3::new(0.0, self.rotation.y, 0.0))
			* na::Matrix4::new_rotation(na::Vector3::new(self.rotation.x, 0.0, 0.0))
	}

	pub fn scaling(&self) -> na::Matrix4<f32> {
		na::Matrix4::new_nonuniform_scaling(&self.scaling)
	}

	pub fn view_projection(&self) -> na::Matrix4<f32> {
		self.projection() * self.view()
	}

	pub fn resize(&mut self, width: f32, height: f32) {
		self.width = width;
		self.height = height;
		let aspect = (3.14 * self.scaling.x) / (FOV * self.scaling.y);
		self.projection = na::Perspective3::new(width / height, aspect, 0.1, 1000.0);
	}

	pub fn size(&self) -> (f32, f32) {
		(self.width, self.height)
	}

	pub fn translate(&mut self, v: &na::Vector3<f32>) {
		let trans = na::Matrix4::new_translation(v);
		let rot = self.rotation();
		let inv_rot = rot.try_inverse().unwrap();
		self.position = rot.transform_point(&(trans * inv_rot).transform_point(&self.position));
	}

	pub fn translate_absolute(&mut self, v: &na::Vector3<f32>) {
		self.position = na::Matrix4::new_translation(v).transform_point(&self.position);
	}

	pub fn rotate(&mut self, v: &na::Vector3<f32>) {
		self.rotation += v;
	}
}
