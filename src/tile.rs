use crate::geometry::{lonlat_to_point, pixel_to_lonlat};
use crate::mesh::Mesh;
use crate::polyline::{Ring, Polyline};
use crate::protos::vector_tile::Tile as VectorTile;
use nalgebra as na;
use std::f32::consts::PI;

const MOVE_TO: u32 = 0x1;
const LINE_TO: u32 = 0x2;
const CLOSE_PATH: u32 = 0x7;

#[derive(Clone, Debug)]
pub struct Tile {
	mesh: Mesh,
	lines: Vec<Polyline>,
}

impl Tile {
	pub fn new() -> Self {
		Self {
			mesh: Mesh::new(),
			lines: vec![],
		}
	}

	pub fn mesh(&self) -> Mesh {
		self.mesh.clone()
	}

	pub fn polylines(&self) -> Vec<Polyline> {
		self.lines.clone()
	}

	pub fn vertices(&self) -> Vec<f32> {
		self.mesh.vertices_as_vec()
	}

	pub fn triangles(&self) -> Vec<u32> {
		self.mesh.triangles_as_vec()
	}

	pub fn from_vector_tile<'a>(raw: VectorTile<'a>, x: i32, y: i32, z: i32) -> Self {
		let mesh = Mesh::new();
		if raw.layers.len() == 0 {
			return Self { mesh, lines: vec![] };
		}

		let layer = &raw.layers[0];
		let extent = layer.extent as f32;
		let mut lines = vec![
			Polyline {
				rings: vec![
					Ring {
						points: vec![
							na::Point3::new(0.0, 0.0, -1.0),
							na::Point3::new(0.1, 0.0, -1.0),
							na::Point3::new(0.2, 0.15, -1.0),
							na::Point3::new(0.1, 0.15, -1.0),
							na::Point3::new(0.0, 0.10, -1.0),
						],
					}
				],
			},
		];
		//return Self { mesh, lines };

		// features
		for feature in &layer.features {
			let mut line = Polyline::from_geometry(&feature.geometry);

			// Convert from texture pixel coords to world coords
			for ring in &mut line.rings {
				for point in &mut ring.points {
					let ll = pixel_to_lonlat(
						&na::Point2::new(x as f32 + point.x / extent, y as f32 + point.y / extent),
						1.0 + z as f32,
					);

					*point = lonlat_to_point(&ll);
				}
			}

			lines.push(line);
		}

		Self { mesh, lines }
	}

	pub fn old_from_vector_tile<'a>(raw: VectorTile<'a>, x: i32, y: i32, z: i32) -> Self {
		let mut mesh = Mesh::new();
		let mut lines = vec![];
		if raw.layers.len() == 0 {
			return Self { mesh, lines: vec![] };
		}

		let layer = &raw.layers[0];
		let extent = layer.extent as f32;

		// features
		for feature in &layer.features {
			let mut edges = vec![];
			let mut geometry = feature.geometry.clone().into_iter();
			let mut cursor = (0.0, 0.0);

			// geometry
			while let Some(cmdint) = geometry.next() {
				let cmd = (cmdint & 0x7) as u32;
				let count = (cmdint >> 3) as u32;

				let make_point = |cursor: (f32, f32)| {
					// pixels coords range from 0.0 to 1.0
					pixel_to_lonlat(
						&na::Point2::new(x as f32 + cursor.0, y as f32 + cursor.1),
						1.0 + z as f32,
					)
				};

				let mut add_edge = |p0: na::Point2<f32>, p1: na::Point2<f32>| {
					let line = p0 - p1;
					let len = line.norm().abs();
					let norm = line.normalize();
					if (norm.x.abs() == 1.0 || norm.y.abs() == 1.0) && len > 3.0 {
						// Weird axis aligned line (borders)
					} else if len > 10.0 {
						// Weird long line ??
					} else if len == 0.0 {
						// Nothing to draw
					} else {
						edges.push((p0, p1));
					}
				};

				let mut line_start = make_point(cursor);
				let mut line_closed = true;
				// command
				for _ in 0..count {
					match cmd {
						MOVE_TO => {
							if !line_closed {
								let p0 = make_point(cursor);
								let p1 = line_start;
								add_edge(p0, p1);
							}

							let param = geometry.next().unwrap() as i32;
							let arg0 = (param >> 1) ^ (-(param & 1));
							let param = geometry.next().unwrap() as i32;
							let arg1 = (param >> 1) ^ (-(param & 1));

							cursor.0 += arg0 as f32 / extent;
							cursor.1 += arg1 as f32 / extent;
							line_start = make_point(cursor);
						}
						LINE_TO => {
							line_closed = false;
							let param = geometry.next().unwrap() as i32;
							let arg0 = (param >> 1) ^ (-(param & 1));
							let param = geometry.next().unwrap() as i32;
							let arg1 = (param >> 1) ^ (-(param & 1));

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

							add_edge(p0, p1);
						}
						CLOSE_PATH => {
							line_closed = true;
							let p0 = make_point(cursor);
							let p1 = line_start;
							add_edge(p0, p1);
						}
						_ => panic!("Unknown command {}", cmd),
					}
				}

				if !line_closed {
					let p0 = make_point(cursor);
					let p1 = line_start;
					add_edge(p0, p1);
				}
			}

			let thickness = 0.01;
			for edge in &edges {
				let p0 = na::Point2::new(edge.0.x, edge.0.y);
				let p1 = na::Point2::new(edge.1.x, edge.1.y);

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

		Self { mesh, lines }
	}
}
