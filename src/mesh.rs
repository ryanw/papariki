use nalgebra as na;

#[derive(Debug, Default, Clone)]
pub struct Mesh {
	pub vertices: Vec<na::Point3<f32>>,
	pub triangles: Vec<(usize, usize, usize)>,
}

impl Mesh {
	pub fn new() -> Self {
		Self {
			vertices: vec![],
			triangles: vec![],
		}
	}

	pub fn cube() -> Self {
		let vertices = vec![
			// Front
			na::Point3::new(-1.0, -1.0,  1.0),
			na::Point3::new( 1.0, -1.0,  1.0),
			na::Point3::new( 1.0,  1.0,  1.0),
			na::Point3::new(-1.0,  1.0,  1.0),
			// Back
			na::Point3::new(-1.0, -1.0, -1.0),
			na::Point3::new( 1.0, -1.0, -1.0),
			na::Point3::new( 1.0,  1.0, -1.0),
			na::Point3::new(-1.0,  1.0, -1.0),
		];

		let triangles = vec![
			// Front
			(0, 1, 2),
			(2, 3, 0),
			// Right
			(1, 5, 6),
			(6, 2, 1),
			// Back
			(7, 6, 5),
			(5, 4, 7),
			// Left
			(4, 0, 3),
			(3, 7, 4),
			// Bottom
			(4, 5, 1),
			(1, 0, 4),
			// Top
			(3, 2, 6),
			(6, 7, 3),
		];
		Self {
			vertices,
			triangles,
		}
	}

	pub fn vertices_as_vec(&self) -> Vec<f32> {
		self.vertices.iter().map(|v| v.iter()).flatten().map(|f| *f).collect()
	}

	pub fn triangles_as_vec(&self) -> Vec<u32> {
		self.triangles
			.iter()
			.map(|v| vec![v.0 as u32, v.1 as u32, v.2 as u32])
			.flatten()
			.collect()
	}

	pub fn vertices(&self) -> &Vec<na::Point3<f32>> {
		&self.vertices
	}

	pub fn vertices_mut(&mut self) -> &mut Vec<na::Point3<f32>> {
		&mut self.vertices
	}

	pub fn triangles(&self) -> &Vec<(usize, usize, usize)> {
		&self.triangles
	}

	pub fn triangles_mut(&mut self) -> &mut Vec<(usize, usize, usize)> {
		&mut self.triangles
	}
}
