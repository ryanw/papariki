use nalgebra as na;

#[derive(Debug, Clone)]
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
