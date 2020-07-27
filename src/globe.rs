use crate::geometry::{LonLat, Mesh};
use crate::protos::vector_tile::Tile as VectorTile;
use crate::wasm;
use crate::{Tile, TileSource, WebTileSource};
use futures::executor::block_on;
use js_sys::{ArrayBuffer, Float32Array, Promise};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{future_to_promise, spawn_local};
use web_sys::{HtmlCanvasElement, HtmlElement, WebGlRenderingContext, WebGlShader};

#[wasm_bindgen]
pub struct Globe {
	tiles: WebTileSource,
	tile_queue: Vec<(i32, i32, i32)>,
}

impl Globe {
	pub fn queue_tile(&mut self, x: i32, y: i32, z: i32) {
		self.tile_queue.push((x, y, z));
	}

	pub async fn update(&mut self) {
		if let Some((x, y, z)) = self.tile_queue.pop() {
			wasm::log(&format!("Fetching tile {:?}", (x, y, z)));
			let tile = self.tiles.get_tile(x, y, z).await;
			wasm::log(&format!("Got tile {:?}", tile));
		}
	}
}

impl Globe {
	pub fn new(token: &str) -> Self {
		Self {
			tile_queue: vec![],
			tiles: WebTileSource::new(token),
		}
	}

	async fn generate_tile(&mut self) -> Result<JsValue, JsValue> {
		let tile = self.tiles.get_tile(0, 0, 0).await;
		wasm::log(&format!("TILES {:?}", tile.vertices()));
		Ok(JsValue::from(true))
	}

	pub async fn get_tiles(&self, ll: &LonLat) -> Vec<Tile> {
		println!("Fetching tile {:?}", ll);

		let zoom = 1;
		let mut tiles = vec![];
		let n = 2i32.pow(zoom);
		for y in 0..n {
			for x in 0..n {
				tiles.push(self.tiles.get_tile(x, y, zoom as i32).await);
			}
		}

		tiles
	}

	pub async fn get_tile(&self, x: i32, y: i32, zoom: i32) -> Tile {
		self.tiles.get_tile(x, y, zoom).await
	}
}
