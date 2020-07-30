use crate::data::WebTileSource;
use crate::geometry::LonLat;
use crate::tile::Tile;

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
			let _tile = self.tiles.get_tile(x, y, z).await;
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
