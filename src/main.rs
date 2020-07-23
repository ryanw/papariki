pub mod protos;
pub mod geometry;

use protos::vector_tile::Tile;
use std::fs;
use std::io::Read;
use flate2::read::GzDecoder;
use quick_protobuf::{MessageRead, BytesReader, Reader};
use reqwest;


fn main() {
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
	let tile = reader.read(|r, b| Tile::from_reader(r, b)).unwrap();

	println!("Hello, world! {:?}", tile.layers[0].name);

}
