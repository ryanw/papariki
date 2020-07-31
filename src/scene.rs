use crate::camera::Camera;
use crate::globe::Globe;
use crate::input::UserInputs;
use crate::mesh::Mesh;
use crate::tile::Tile;
use crate::geometry::{lonlat_to_point, pixel_to_lonlat};
use nalgebra as na;
use std::collections::HashMap;
use std::f32::consts::PI;
use std::{cell::RefCell, rc::Rc};
use crate::wasm;

type TileCoord = (i32, i32, i32);

fn rad_to_deg(rad: f32) -> f32 {
	((rad * (180.0 / PI) + 180.0) % 360.0) - 180.0
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
}

impl Scene {
	pub fn new(globe: Rc<RefCell<Globe>>) -> Self {
		Self {
			globe,
			tiles: HashMap::new(),
			prev_mouse_position: None,
			prev_wheel_position: Some((0.0, 0.0)),
			zoom: 1.0,
			..Self::default()
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
		let id = self.add(SceneItem {
			mesh: Mesh::cube(),
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

	pub fn tick(&mut self, dt: f64, inputs: &UserInputs) {
		self.update_tiles();

		// Update mousewheel zooming
		let dy = (inputs.wheel_position().1 - self.prev_wheel_position.as_ref().unwrap().1) as f32;
		if dy != 0.0 {
			self.zoom -= dy / 15.0;

			if self.zoom > 2.98 {
				self.zoom = 2.98;
			}
			if self.zoom < 0.33 {
				self.zoom = 0.33;
			}
		}
		self.prev_wheel_position = Some(inputs.wheel_position());

		// Update rotation
		if inputs.is_mouse_down() {
			if self.prev_mouse_position.is_none() {
				// Mouse down
				self.prev_mouse_position = Some(inputs.mouse_position());
				// Coordinate at center of screen
				let lat = rad_to_deg(self.globe_rotation.x);
				let lon = rad_to_deg(self.globe_rotation.y);
				self.add_marker(na::Point2::new(lon, lat));
			} else {
				// Mouse move
				let dx = (inputs.mouse_position().0 - self.prev_mouse_position.as_ref().unwrap().0) as f32;
				let dy = (inputs.mouse_position().1 - self.prev_mouse_position.as_ref().unwrap().1) as f32;
				self.globe_rotation.x += dy * 0.0025 / self.zoom;
				self.globe_rotation.y += dx * -0.0025 / self.zoom;
				if self.zoom <= 0.0 {
					panic!("Invalid zoom");
				}

				let l = PI * 0.4;
				if self.globe_rotation.x > l {
					self.globe_rotation.x = l
				}
				if self.globe_rotation.x < -l {
					self.globe_rotation.x = -l
				}

				// Save mouse position for next time
				if self.prev_mouse_position.is_some() {
					self.prev_mouse_position = Some(inputs.mouse_position());
				}

			}
		} else {
			// Mouse up
			self.prev_mouse_position = None;
			self.globe_rotation.y += -0.5 * dt as f32;
		}

		// Update camera
		self.camera.position = na::Point3::new(0.0, 0.0, self.zoom - 3.7);

		// Update tile mesh rotation
		let model = na::Matrix4::from_euler_angles(self.globe_rotation.x, 0.0, 0.0)
			* na::Matrix4::from_euler_angles(0.0, self.globe_rotation.y, 0.0);

		for (coord, item_id) in &mut self.tiles {
			let item = &mut self.items[*item_id];
			item.transform = model;
		}

		// HACK update the cube
		for (item_id, lonlat) in &self.markers {
			let pos = lonlat_to_point(&lonlat);
			self.items[*item_id].transform = model * na::Matrix4::new_translation(&pos.coords) * na::Matrix4::new_scaling(0.01);
		}
	}
}
