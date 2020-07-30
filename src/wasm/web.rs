use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{JsValue, JsCast, convert::FromWasmAbi};
use js_sys::Function;
use web_sys::{MouseEvent, EventTarget, HtmlElement};

pub fn add_event_listener<E: 'static + FromWasmAbi, F: 'static + FnMut(E)>(target: &EventTarget, event: &str, callback: F) -> Result<(), JsValue> {
	let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(_)>);
	let result = target.add_event_listener_with_callback(event, callback.as_ref().unchecked_ref());
	callback.forget();
	result
}
