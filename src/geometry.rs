use crate::mesh::Mesh;
use crate::protos::vector_tile::mod_Tile::Feature;
use nalgebra as na;

#[derive(Debug, Clone)]
pub struct LonLat(na::Point2<f32>);

impl LonLat {
	pub fn new(lon: f32, lat: f32) -> Self {
		Self(na::Point2::new(lon, lat))
	}
}

impl Feature {
	pub fn to_mesh(&self) -> Mesh {
		Mesh {
			vertices: vec![],
			triangles: vec![],
		}
	}
}
