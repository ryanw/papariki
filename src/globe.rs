use crate::geometry::{Mesh, LonLat};
use crate::protos::vector_tile::{Tile as VectorTile};
use crate::{WebTileSource, Tile, TileSource};


pub struct Globe {
	tiles: WebTileSource,
}

impl Globe {
	pub fn new() -> Self {
		Self {
			tiles: WebTileSource::new(),
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
