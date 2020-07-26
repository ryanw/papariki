use wasm_bindgen::prelude::*;

use nalgebra as na;
use std::f32::consts::PI;
use crate::geometry::{Mesh, LonLat};
use crate::protos::vector_tile::{Tile as VectorTile};

#[cfg(target_arch = "wasm32")]
use crate::wasm;

const MOVE_TO: u32 = 0x1;
const LINE_TO: u32 = 0x2;
const CLOSE_PATH: u32 = 0x7;

#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct Tile {
	mesh: Mesh,
}

#[wasm_bindgen]
impl Tile {
	pub fn new() -> Self {
		Self {
			mesh: Mesh::new(),
		}
	}

	pub fn mesh(&self) -> Mesh {
		self.mesh.clone()
	}

	#[wasm_bindgen(js_name="toString")]
	pub fn to_string(&self) -> String {
		format!("{:?}", self)
	}

	#[wasm_bindgen]
	pub fn vertices(&self) -> Vec<f32> {
		self.mesh.vertices_as_vec()
	}

	#[wasm_bindgen]
	pub fn triangles(&self) -> Vec<u32> {
		self.mesh.triangles_as_vec()
	}
}

impl Tile {

	pub fn from_vector_tile<'a>(raw: VectorTile<'a>, x: i32, y: i32, z: i32) -> Self {
		let mut mesh = Mesh::new();

		let layer = &raw.layers[0];
		let extent = layer.extent as f32;

		'feature: for feature in &layer.features {
			let mut edges = vec![];
			let mut geometry = feature.geometry.clone();
			let mut cursor = (0.0, 0.0);

			'geom: while geometry.len() > 0 {
				let cmdint = geometry.remove(0) as i32;
				let cmd = (cmdint & 0x7) as u32;
				let count = (cmdint >> 3) as u32;

				let make_point = |cursor: (f32, f32)| {
					// pixels coords range from 0.0 to 1.0
					pixel_to_lonlat(&na::Point2::new(x as f32 + cursor.0, y as f32 + cursor.1), 1.0 + z as f32)
				};

				let mut line_start = make_point(cursor);
				let mut line_closed = true;
				'cmd: for i in 0..count {
					match cmd {
						MOVE_TO => {
							if !line_closed {
								let p0 = make_point(cursor);
								let p1 = line_start;
								edges.push((p0, p1));
							}

							let param = geometry.remove(0) as i32;
							let arg0 = ((param >> 1) ^ (-(param & 1)));
							let param = geometry.remove(0) as i32;
							let arg1 = ((param >> 1) ^ (-(param & 1)));

							cursor.0 += arg0 as f32 / extent;
							cursor.1 += arg1 as f32 / extent;
							line_start = make_point(cursor);
						}
						LINE_TO => {
							line_closed = false;
							let param = geometry.remove(0) as i32;
							let arg0 = ((param >> 1) ^ (-(param & 1)));
							let param = geometry.remove(0) as i32;
							let arg1 = ((param >> 1) ^ (-(param & 1)));

							let p0 = make_point(cursor);

							cursor.0 += arg0 as f32 / extent;
							cursor.1 += arg1 as f32 / extent;

							let p1 = make_point(cursor);

							let border = 0.0;

							if p0.x >= 180.0 - border || p1.x >= 180.0 - border {
								continue;
							}
							if p0.x <= -180.0 + border || p1.x <= -180.0 + border {
								continue;
							}
							if p0.y >= 90.0 - border || p1.y >= 90.0 - border {
								continue;
							}
							if p0.y <= -90.0 + border || p1.y <= -90.0 + border {
								continue;
							}

							// FIXME Hack to remove glitchy lines
							let line = (p0 - p1);
							let norm = line.normalize();
							if (norm.x.abs() == 1.0 || norm.y.abs() == 1.0) && line.norm().abs() > 3.0 {
								//wasm::log(&format!("Glitchy {:?} - {:?} = {:?} / {:?}", p0, p1, line.norm(), line.normalize()));
							}
							else if line.norm().abs() > 10.0 {
								//wasm::log(&format!("derp {:?} - {:?} = {:?} / {:?}", p0, p1, line.norm(), line.normalize()));
							}
							else {
								//wasm::log(&format!("p {:?} - {:?} = {:?} / {:?}", p0, p1, line.norm(), line.normalize()));
								edges.push((p0, p1));
							}
						}
						CLOSE_PATH => {
							line_closed = true;
							if edges.len() > 0 {
								let p0 = make_point(cursor);
								let p1 = line_start;
								edges.push((p0, p1));
							}
						}
						_ => panic!("Unknown command {}", cmd),
					}
				}

				if !line_closed {
					if edges.len() > 0 {
						let p0 = make_point(cursor);
						let p1 = line_start;
						if (p0 - p1).norm().abs() < 4.0 {
							edges.push((p0, p1));
						}
					}
				}
			}

			let thickness = 0.1;
			for edge in &edges {
				let p0 = na::Point2::new(-edge.0.x, -edge.0.y);
				let p1 = na::Point2::new(-edge.1.x, -edge.1.y);

				let dir = na::Matrix3::new_rotation(PI / 2.0).transform_vector(&((p0 - p1).normalize() * thickness));
				let mat = na::Matrix3::new_translation(&dir);
				let p2 = mat.transform_point(&p0);
				let p3 = mat.transform_point(&p1);
				let p0 = mat.try_inverse().unwrap().transform_point(&p0);
				let p1 = mat.try_inverse().unwrap().transform_point(&p1);
				mesh.vertices_mut().push(lonlat_to_point(&p0));
				mesh.vertices_mut().push(lonlat_to_point(&p1));
				mesh.vertices_mut().push(lonlat_to_point(&p2));
				mesh.vertices_mut().push(lonlat_to_point(&p3));
			}
			let step = 4;
			for i in 0..edges.len() {
				let p0 = i * step;
				let p1 = i * step + 1;
				let p2 = i * step + 2;
				let p3 = i * step + 3;
				mesh.triangles_mut().push((p0, p1, p2));
				mesh.triangles_mut().push((p1, p3, p2));
				mesh.triangles_mut().push((p2, p1, p0));
				mesh.triangles_mut().push((p2, p3, p1));
			}
		}

		Self {
			mesh,
		}
	}
}

fn lonlat_to_point(ll: &na::Point2<f32>) -> na::Point3<f32> {
	let rad = 1.005;
	let lon = (ll.x).to_radians() as f32;
	let lat = (ll.y - 90.0).to_radians() as f32;

	na::Point3::new(
		-rad * lat.sin() * lon.sin(),
		-rad * lat.cos(),
		rad * lat.sin() * lon.cos(),
	)
}

fn pixel_to_lonlat(p: &na::Point2<f32>, zoom: f32) -> na::Point2<f32> {
	let tile_size = 0.5f32;
	let c = tile_size * 2.0_f32.powi(zoom as i32);
	let bc = c / 360.0;
	let cc = c / (2.0 * PI);

	let e = c / 2.0;
	let lon = (p.x - e) / bc;
	let g = (p.y - e) / -cc;
	let lat = (2.0f32 * g.exp().atan() - 0.5 * PI).to_degrees();

	na::Point2::new(lon, lat)
}
