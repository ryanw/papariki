use crate::wasm;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::Closure;
use wasm_bindgen::{convert::FromWasmAbi, JsCast, JsValue};
use web_sys::EventTarget;

pub type AnimateLoop = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

// Add a javascript event listener to a javascript object
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

// Runs closure on every requestAnimationFrame
pub fn request_animation_loop<F: 'static + FnMut()>(mut callback: F) -> AnimateLoop {
	let animate_callback: AnimateLoop = Rc::new(RefCell::new(None));
	let closure = {
		let animate = animate_callback.clone();
		Closure::wrap(Box::new(move || {
			callback();
			if let Ok(callback) = animate.try_borrow() {
				if let Some(callback) = &*callback {
					let window = web_sys::window().unwrap();
					let _animation_id = window
						.request_animation_frame(callback.as_ref().unchecked_ref())
						.unwrap();
				}
			} else {
				wasm::log("Failed to borrow animate frame closure");
			}
		}) as Box<dyn FnMut()>)
	};

	let window = web_sys::window().unwrap();
	window
		.request_animation_frame(closure.as_ref().unchecked_ref())
		.unwrap();

	*animate_callback.borrow_mut() = Some(closure);

	animate_callback
}
