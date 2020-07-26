use wasm_bindgen::prelude::*;
use crate::{Globe, Tile, LonLat};
use js_sys::Array;
use wasm_bindgen::JsValue;

// Imported from JS land
#[wasm_bindgen]
extern {
	#[wasm_bindgen(js_namespace = console)]
	pub fn log(s: &str);

	pub type Performance;
	#[wasm_bindgen(js_class = performance, static_method_of = Performance)]
	pub fn now() -> f64;
}

// Export to JS land
#[wasm_bindgen(js_name=getTiles)]
pub async fn get_tiles(lon: f32, lat: f32) -> Array {
	let mut globe = Globe::new();
	let ll = LonLat::new(0.0, 52.0);
	let tiles = globe.get_tiles(&ll).await;
	tiles.into_iter().map(JsValue::from).collect()
}
