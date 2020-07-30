
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{convert::FromWasmAbi, JsCast, JsValue};
use web_sys::{EventTarget};

pub fn add_event_listener<E: 'static + FromWasmAbi, F: 'static + FnMut(E)>(
	target: &EventTarget,
	event: &str,
	callback: F,
) -> Result<(), JsValue> {
	let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>);
	let result = target.add_event_listener_with_callback(event, callback.as_ref().unchecked_ref());
	callback.forget();
	result
}
