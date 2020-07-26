use crate::Tile;
use crate::protos::vector_tile::{Tile as VectorTile};

use std::fs;
use std::io::Read;
use std::future::Future;
use flate2::read::GzDecoder;
use quick_protobuf::{MessageRead, BytesReader, Reader};

use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, RequestMode, Response};
use js_sys::{Uint8Array, ArrayBuffer};

#[cfg(not(target_arch = "wasm32"))]
use ureq;

#[cfg(target_arch = "wasm32")]
use crate::wasm;

pub trait TileSource {
	fn get_tile(&self, x: i32, y: i32, z: i32) -> Tile;
}

#[derive(Debug, Default)]
pub struct WebTileSource {
}

impl WebTileSource {
	pub fn get_url(&self, x: i32, y: i32, z: i32) -> String {
		let token = "pk.eyJ1IjoicmF0aWNpZGUiLCJhIjoiY2p1cmwxOWt2MDBkMjN6cGU5M2wyeDdyMCJ9.v3zWLlTBAHww7nYTZbTgLA";
		format!("https://api.mapbox.com/v4/mapbox.mapbox-streets-v8/{}/{}/{}.vector.pbf?access_token={}", z, x, y, token)
		//format!("http://localhost:8880/data/v3/{}/{}/{}.pbf", z, x, y)
	}

	#[cfg(target_arch = "wasm32")]
	pub async fn get_tile(&self, x: i32, y: i32, z: i32) -> Tile {
		wasm::log(&format!("Rust getting tile {}x{}x{}", x, y, z));
		// Use 'fetch' from JS
		let url = self.get_url(x, y, z);
		wasm::log(&format!("URL: {}", url));

		let mut opts = RequestInit::new();
		opts.method("GET");
		opts.mode(RequestMode::Cors);
		let request = Request::new_with_str_and_init(&url, &opts).unwrap();
		let window = web_sys::window().unwrap();
		let resp_value = JsFuture::from(window.fetch_with_request(&request)).await.unwrap();
		let resp: Response = resp_value.dyn_into().unwrap();
		let body: ArrayBuffer = JsFuture::from(resp.array_buffer().unwrap()).await.unwrap().dyn_into().unwrap();
		let bytes = Uint8Array::new(&body).to_vec();
		wasm::log(&format!("Hello, world! {:?}", bytes));

		// Decode PBF
		let mut reader = Reader::from_bytes(bytes);
		let vt = reader.read(|r, b| VectorTile::from_reader(r, b)).unwrap();
		Tile::from_vector_tile(vt, x, y, z)
	}

	#[cfg(not(target_arch = "wasm32"))]
	pub async fn get_tile(&self, x: i32, y: i32, z: i32) -> Tile {
		// Read from web
		let mut res = ureq::get(&self.get_url(x, y, z)).call().into_reader();
		let mut gz_pbf = vec![];
		res.read_to_end(&mut gz_pbf);

		// Decode gzip
		let mut pbf = GzDecoder::new(&*gz_pbf);
		let mut bytes = vec![];
		pbf.read_to_end(&mut bytes);

		// Decode PBF
		let mut reader = Reader::from_bytes(bytes);
		let vt = reader.read(|r, b| VectorTile::from_reader(r, b)).unwrap();
		println!("Hello, world! {:?}", vt);
		Tile::from_vector_tile(vt, x, y, z)
	}
}

impl WebTileSource {
	pub fn new() -> Self {
		Self {
		}
	}
}
