use nalgebra as na;
use std::f32::consts::PI;
use crate::geometry::{Mesh, LonLat};
use crate::protos::vector_tile::{Tile as VectorTile};

const MOVE_TO: u32 = 0x1;
const LINE_TO: u32 = 0x2;
const CLOSE_PATH: u32 = 0x7;

#[derive(Clone, Debug)]
pub struct Tile {
}

impl Tile {
	pub fn from_vector_tile<'a>(raw: VectorTile<'a>) -> Self {
		let x = 0;
		let y = 0;
		let z = 0;


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
				'cmd: for i in 0..count {
					match cmd {
						MOVE_TO => {
							let param = geometry.remove(0) as i32;
							let arg0 = ((param >> 1) ^ (-(param & 1)));
							let param = geometry.remove(0) as i32;
							let arg1 = ((param >> 1) ^ (-(param & 1)));

							cursor.0 += arg0 as f32 / extent;
							cursor.1 += arg1 as f32 / extent;
							line_start = make_point(cursor);
						}
						LINE_TO => {
							let param = geometry.remove(0) as i32;
							let arg0 = ((param >> 1) ^ (-(param & 1)));
							let param = geometry.remove(0) as i32;
							let arg1 = ((param >> 1) ^ (-(param & 1)));

							let p0 = make_point(cursor);

							cursor.0 += arg0 as f32 / extent;
							cursor.1 += arg1 as f32 / extent;

							let p1 = make_point(cursor);

							let border = 10.0;

							if p0.x >= 180.0 - border && p1.x >= 180.0 - border {
								continue;
							}
							if p0.x <= -180.0 + border && p1.x <= -180.0 + border {
								continue;
							}
							if p0.y >= 90.0 - border && p1.y >= 90.0 - border {
								continue;
							}
							if p0.y <= -90.0 + border && p1.y <= -90.0 + border {
								continue;
							}
							edges.push((p0, p1));
						}
						CLOSE_PATH => {
							if edges.len() > 0 {
								let p0 = make_point(cursor);
								let p1 = line_start;
								edges.push((p0, p1));
							}
						}
						_ => panic!("Unknown command {}", cmd),
					}
				}
			}

			/*
			   let mut mesh = StaticMesh::new();
			   for p in &edges {
			   mesh.vertices.push(lonlat_to_point(&p.0));
			   mesh.vertices.push(lonlat_to_point(&p.1));
			   }
			   for i in 0..edges.len() {
			   mesh.triangles.push((i * 2, i * 2, i * 2 + 1));
			   mesh.triangles.push((i * 2, i * 2, i * 2 + 1));
			   mesh.triangles.push((i * 2, i * 2, i * 2 + 1));
			   mesh.triangles.push((i * 2, i * 2, i * 2 + 1));
			   }
			   self.tiles.push(mesh);
			   */
		}

		Self {
		}
	}

	pub fn as_mesh(&self) -> Mesh {
		Mesh::new()
	}
}

fn lonlat_to_point(ll: &na::Point2<f32>) -> na::Point3<f32> {
	let rad = 1.03;
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
