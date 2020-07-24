use crate::geometry::{Mesh, LonLat};
use crate::protos::vector_tile::{Tile as VectorTile};
use crate::Tile;

use std::fs;
use std::io::Read;
use flate2::read::GzDecoder;
use quick_protobuf::{MessageRead, BytesReader, Reader};
use reqwest;


#[derive(Clone, Debug)]
pub struct Globe {
}

impl Globe {
	pub fn new() -> Self {
		Self {
		}
	}

	pub fn get_tile(&self, ll: &LonLat) -> Tile {
		// Read from web
		let mut res = reqwest::blocking::get("http://localhost:8880/data/v3/0/0/0.pbf").unwrap();
		let mut gz_pbf = vec![];
		res.read_to_end(&mut gz_pbf);

		// Decode gzip
		let mut pbf = GzDecoder::new(&*gz_pbf);
		let mut bytes = vec![];
		pbf.read_to_end(&mut bytes);

		// Decode PBF
		let mut reader = Reader::from_bytes(bytes);
		Tile::from_vector_tile(reader.read(|r, b| VectorTile::from_reader(r, b)).unwrap())
	}
}
