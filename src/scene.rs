use crate::camera::Camera;
use crate::globe::Globe;
use crate::input::UserInputs;
use crate::mesh::Mesh;
use crate::tile::Tile;
use crate::geometry::{point_to_lonlat, lonlat_to_point, pixel_to_lonlat};
use nalgebra as na;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::{cell::RefCell, rc::Rc};
use crate::wasm;

type TileCoord = (i32, i32, i32);

fn rad_to_deg(rad: f32) -> f32 {
	((rad * (180.0 / PI) + 180.0) % 360.0) - 180.0
}

fn map_range(val: f32, min0: f32, max0: f32, min1: f32, max1: f32) -> f32 {
	(val - min0) * (max1 - min1) / (max0 - min0) + min1
}

fn ray_sphere_intersection(center: &na::Point3<f32>, radius: f32, origin: &na::Point3<f32>, direction: &na::Vector3<f32>) -> Option<na::Point3<f32>> {
	let mut p = na::Point3::new(
		origin.x - center.x,
		origin.y - center.y,
		origin.z - center.z,
	);

	let a = direction.norm().powf(2.0);
	let b = direction.dot(&p.coords);
	let c = p.coords.norm().powf(2.0) - radius * radius;
	let delta = b * b - a * c;

	if delta < 0.0 {
		return None;
	}

	let delta_sqrt = delta.sqrt();

	let tmin = (-b - delta_sqrt) / a;
	let tmax = (-b + delta_sqrt) / a;

	if tmax < 0.0 {
		return None
	}

	let t = if tmin >= 0.0 { tmin } else { tmax };

	p.x = origin.x + t * direction.x;
	p.y = origin.y + t * direction.y;
	p.z = origin.z + t * direction.z;

	Some(p)
}

#[derive(Debug, Default)]
pub struct SceneItem {
	pub mesh: Mesh,
	pub transform: na::Matrix4<f32>,
	pub version: usize,
}

#[derive(Debug, Default)]
pub struct Scene {
	items: Vec<SceneItem>,
	markers: HashMap<usize, na::Point2<f32>>,
	tiles: HashMap<TileCoord, usize>,
	camera: Camera,
	globe: Rc<RefCell<Globe>>,
	globe_rotation: na::Vector3<f32>,
	prev_mouse_position: Option<(i32, i32)>,
	prev_wheel_position: Option<(f32, f32)>,
	zoom: f32,
	clicking: bool,
}

impl Scene {
	pub fn new(globe: Rc<RefCell<Globe>>) -> Self {
		Self {
			globe,
			tiles: HashMap::new(),
			prev_mouse_position: None,
			prev_wheel_position: Some((0.0, 0.0)),
			zoom: 0.3,
			clicking: false,
			..Self::default()
		}
	}

	fn on_click(&mut self, pos: (i32, i32)) {
		// Coordinate at center of screen
		let lat = rad_to_deg(self.globe_rotation.x);
		let lon = rad_to_deg(self.globe_rotation.y);

		// Viewport -> camera
		let (w, h) = self.camera.size();
		let x = ((pos.0 * 2) as f32 - w) / w;
		let y = -((pos.1 * 2) as f32 - h) / h;

		// Camera -> world
		let inv_view = self.camera.view().try_inverse().unwrap();
		let inv_proj = self.camera.projection().try_inverse().unwrap();
		let vp = self.camera.view_projection().try_inverse().unwrap();
		let pv = inv_view * inv_proj;

		// Transform origin/dir from screen to world space
		let mut origin = vp.transform_point(&na::Point3::new(x, y, -1.0));
		//let mut origin = inv_view.transform_point(&na::Point3::new(x, y, -0.1));
		let dest       = vp.transform_point(&na::Point3::new(x, y, 0.0));
		let dir = (dest - origin).normalize();
		//origin.z += self.camera.near;

		let intersection = ray_sphere_intersection(
			&na::Point3::new(0.0, 0.0, 0.0),
			self.scale(),
			&origin,
			&dir,
		);

		// Put a little cube where they clicked
		if intersection.is_some() {
			let model = na::Matrix4::from_euler_angles(0.0, -self.globe_rotation.y, 0.0)
			* na::Matrix4::from_euler_angles(-self.globe_rotation.x, 0.0, 0.0);
			let intersection = model.transform_point(&intersection.unwrap());
			let ll = point_to_lonlat(&intersection);
			self.add_marker(ll);
		}
	}

	fn on_mouse_down(&mut self, pos: (i32, i32)) {
		self.clicking = true;
	}

	fn on_mouse_up(&mut self, pos: (i32, i32)) {
		if self.clicking {
			self.on_click(pos);
			self.clicking = false;
		}

	}

	fn on_mouse_move(&mut self, pos: (i32, i32)) {
		if pos == self.prev_mouse_position.unwrap() {
			return;
		}
		self.clicking = false;
		// Mouse move
		let dx = (pos.0 - self.prev_mouse_position.as_ref().unwrap().0) as f32;
		let dy = (pos.1 - self.prev_mouse_position.as_ref().unwrap().1) as f32;
		let z = 0.01 / (1.0 + self.zoom);
		self.globe_rotation.x += dy * 0.3 * z;
		self.globe_rotation.y += dx * -0.3 * z;

		let l = PI * 0.4;
		if self.globe_rotation.x > l {
			self.globe_rotation.x = l
		}
		if self.globe_rotation.x < -l {
			self.globe_rotation.x = -l
		}

		// Save mouse position for next time
		if self.prev_mouse_position.is_some() {
			self.prev_mouse_position = Some(pos);
		}

	}

	fn load_tile(&mut self, x: i32, y: i32, z: i32) {
		if let Ok(mut globe) = self.globe.try_borrow_mut() {
			globe.queue_tile(x, y, z);
		}
	}

	pub fn camera(&self) -> &Camera {
		&self.camera
	}

	pub fn camera_mut(&mut self) -> &mut Camera {
		&mut self.camera
	}

	pub fn add(&mut self, item: SceneItem) -> usize {
		let id = self.items.len();
		self.items.push(item);
		id
	}

	pub fn add_marker(&mut self, lonlat: na::Point2<f32>) {
		wasm::log(&format!("Adding marker {:?}", lonlat));
		let id = self.add(SceneItem {
			mesh: Mesh::cube(1.0),
			transform: na::Matrix4::identity(),
			version: 0,
		});
		self.markers.insert(id, lonlat);
	}

	pub fn items(&self) -> &Vec<SceneItem> {
		&self.items
	}

	pub fn items_mut(&mut self) -> &mut Vec<SceneItem> {
		&mut self.items
	}

	pub fn build_next_tile(&mut self) {
		if let Ok(globe) = self.globe.try_borrow() {
		}
	}

	pub fn globe_rc(&self) -> Rc<RefCell<Globe>> {
		self.globe.clone()
	}

	pub fn update_tiles(&mut self) {
		if let Ok(globe) = self.globe.try_borrow() {
			for (coord, tile) in globe.tiles() {
				if !self.tiles.contains_key(&coord) {
					let idx = self.items.len();
					self.items.push(SceneItem {
						mesh: tile.mesh(),
						transform: na::Matrix4::identity(),
						version: 0,
					});
					self.tiles.insert(coord.clone(), idx);
				}
			}
		}
	}

	pub fn scale(&self) -> f32 {
		map_range(self.zoom.powf(2.0), 0.0, 1.0, 0.5, 1.9)
	}

	pub fn tick(&mut self, dt: f64, inputs: &UserInputs) {
		self.update_tiles();

		// Update mousewheel zooming
		let dy = -(inputs.wheel_position().1 - self.prev_wheel_position.as_ref().unwrap().1) as f32;
		if dy != 0.0 {
			self.zoom += dy * 0.03;
			if self.zoom > 1.0 {
				self.zoom = 1.0;
			}
			if self.zoom < 0.0 {
				self.zoom = 0.0;
			}
		}
		self.prev_wheel_position = Some(inputs.wheel_position());

		// Update mouse events
		if inputs.is_mouse_down() {
			if self.prev_mouse_position.is_none() {
				self.prev_mouse_position = Some(inputs.mouse_position());
				self.on_mouse_down(inputs.mouse_position());
			} else {
				self.on_mouse_move(inputs.mouse_position());

				if self.prev_mouse_position.is_some() {
					self.prev_mouse_position = Some(inputs.mouse_position());
				}
			}
		} else {
			self.on_mouse_up(inputs.mouse_position());
			self.prev_mouse_position = None;

			self.globe_rotation.y += dt as f32 * -0.3;
		}

		// Update tile mesh rotation and scale
		let model = na::Matrix4::from_euler_angles(self.globe_rotation.x, 0.0, 0.0)
			* na::Matrix4::from_euler_angles(0.0, self.globe_rotation.y, 0.0)
			* na::Matrix4::new_scaling(self.scale());

		// Update camera
		self.camera.position = na::Point3::new(0.0, 0.0, -2.0);


		for (coord, item_id) in &mut self.tiles {
			let item = &mut self.items[*item_id];
			item.transform = model;
		}

		// Update the markers
		for (item_id, lonlat) in &self.markers {
			let pos = lonlat_to_point(&lonlat);
			let item = &mut self.items[*item_id];
			item.transform = model * na::Matrix4::new_translation(&pos.coords) * na::Matrix4::new_scaling(0.01);
		}
	}
}
