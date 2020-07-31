use crate::data::WebTileSource;
use crate::geometry::LonLat;
use crate::tile::Tile;
use std::collections::HashMap;

type TileCoord = (i32, i32, i32);

#[derive(Debug, Default)]
pub struct Globe {
	source: WebTileSource,
	tile_queue: Vec<(i32, i32, i32)>,
	tiles: HashMap<TileCoord, Tile>,
}

impl Globe {
	pub fn new(token: &str) -> Self {
		Self {
			tile_queue: vec![],
			tiles: HashMap::default(),
			source: WebTileSource::new(token),
		}
	}

	pub fn tiles(&self) -> &HashMap<TileCoord, Tile> {
		&self.tiles
	}

	pub fn queue_tile(&mut self, x: i32, y: i32, z: i32) {
		self.tile_queue.push((x, y, z));
	}

	pub async fn update(&mut self) {
		if let Some(coord) = self.tile_queue.pop() {
			let tile = self.source.get_tile(coord.0, coord.1, coord.2).await;
			self.tiles.insert(coord, tile);
		}
	}

	pub async fn get_tiles(&self, ll: &LonLat) -> Vec<Tile> {
		println!("Fetching tile {:?}", ll);

		let zoom = 1;
		let mut tiles = vec![];
		let n = 2i32.pow(zoom);
		for y in 0..n {
			for x in 0..n {
				tiles.push(self.source.get_tile(x, y, zoom as i32).await);
			}
		}

		tiles
	}

	pub async fn get_tile(&self, x: i32, y: i32, zoom: i32) -> Tile {
		self.source.get_tile(x, y, zoom).await
	}

	pub async fn load_tile(&mut self, x: i32, y: i32, z: i32) {
		let tile = self.source.get_tile(x, y, z).await;
		// TODO something with tile
	}
}
