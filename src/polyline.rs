use nalgebra as na;

const MOVE_TO: i32 = 0x1;
const LINE_TO: i32 = 0x2;
const CLOSE_PATH: i32 = 0x7;

enum Command {
	MoveTo(i32, i32),
	LineTo(i32, i32),
	ClosePath,
}

struct GeometryParser<'a> {
	cursor: (i32, i32),
	repeating: u32,
	current_command: Option<i32>,
	data: std::slice::Iter<'a, u32>,
}

impl<'a> GeometryParser<'a> {
	pub fn new(data: &'a [u32]) -> Self {
		Self {
			cursor: (0, 0),
			repeating: 0,
			current_command: None,
			data: data.iter(),
		}
	}

	fn read_param(&mut self) -> i32 {
		let param = *self.data.next().unwrap() as i32;
		(param >> 1) ^ (-(param & 1))
	}
}

impl<'a> std::iter::Iterator for GeometryParser<'a> {
	type Item = Command;

	fn next(&mut self) -> Option<Self::Item> {
		let cmd = {
			if self.repeating == 0 {
				if let Some(cmdint) = self.data.next() {
					let cmd = (cmdint & 0x7) as i32;
					let count = (cmdint >> 3) as u32;
					self.current_command = Some(cmd);
					self.repeating = count;
				}
				else {
					return None;
				}
			}

			self.current_command.unwrap()
		};

		if self.repeating == 0 {
			return None;
		}

		self.repeating -= 1;

		match cmd {
			MOVE_TO => {
				self.cursor.0 += self.read_param();
				self.cursor.1 += self.read_param();
				return Some(Command::MoveTo(self.cursor.0, self.cursor.1));
			}
			LINE_TO => {
				self.cursor.0 += self.read_param();
				self.cursor.1 += self.read_param();
				return Some(Command::LineTo(self.cursor.0, self.cursor.1));
			}
			CLOSE_PATH => {
				return Some(Command::ClosePath);
			}
			_ => panic!("Unknown command {}", cmd),
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct Ring {
	pub points: Vec<na::Point3<f32>>,
}

impl Ring {
	pub fn close(&mut self) {
		if self.points.len() > 1 {
			//self.points.push(self.points[0].clone());
		}
	}
}

#[derive(Debug, Default, Clone)]
pub struct Polyline {
	pub rings: Vec<Ring>,
}

impl Polyline {
	pub fn from_geometry(geom: &[u32]) -> Self {
		let mut geometry = GeometryParser::new(geom);
		let mut rings = vec![];
		let mut ring = Ring { points: vec![] };

		let min = 1;

		// Parse the geometry
		for cmd in geometry {
			match cmd {
				Command::MoveTo(x, y) => {
					if ring.points.len() > min {
						ring.close();
						rings.push(ring);
					}
					ring = Ring { points: vec![na::Point3::new(x as f32, y as f32, 0.0)] };
				}
				Command::LineTo(x, y) => {
					ring.points.push(na::Point3::new(x as f32, y as f32, 0.0));
				}
				Command::ClosePath => {
					if ring.points.len() > min {
						ring.close();
						rings.push(ring);
					}
					ring = Ring { points: vec![] };
				}
			}
		}

		if ring.points.len() > min {
			ring.close();
			rings.push(ring);
		}

		Self { rings }
	}
}
