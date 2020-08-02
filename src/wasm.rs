mod renderer;
mod glmesh;
mod web;
mod input;

pub use glmesh::GlMesh;
pub use renderer::WebGlRenderer;
pub use input::HtmlInputs;
use crate::globe::Globe;
use crate::scene::{Scene, SceneItem};
use crate::mesh::Mesh;
use nalgebra as na;

use std::panic;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::future_to_promise;
use web_sys::{self, HtmlElement};

// Imported from JS land
#[wasm_bindgen]
extern "C" {
	#[wasm_bindgen(js_namespace=console)]
	pub fn log(s: &str);
}

pub fn now() -> f64 {
	web_sys::window().unwrap().performance().unwrap().now()
}

// Export to JS land
#[wasm_bindgen]
pub struct Environment {
	scene: Rc<RefCell<Scene>>,
	renderer: Rc<RefCell<WebGlRenderer>>,
	inputs: Rc<RefCell<HtmlInputs>>,
	animate_loop: web::AnimateLoop,
}

#[wasm_bindgen]
pub fn attach(container: &HtmlElement, token: &str) -> Environment {
	panic::set_hook(Box::new(console_error_panic_hook::hook));

	let globe = Rc::new(RefCell::new(Globe::new(token)));

	let mut env = Environment {
		scene: Rc::new(RefCell::new(Scene::new(globe.clone()))),
		inputs: Rc::new(RefCell::new(HtmlInputs::default())),
		renderer: Rc::new(RefCell::new(WebGlRenderer::new(1024, 768))),
		animate_loop: Rc::new(RefCell::new(None)),
	};
	// Add renderer to the DOM
	{
		let mut renderer = env.renderer.borrow_mut();
		renderer.attach(container);
		let mut inputs = env.inputs.borrow_mut();
		inputs.attach(container);
	}

	// Run on next tick
	let _tile_promise = future_to_promise({
		let renderer = env.renderer.clone();
		let scene = env.scene.clone();

		async move {
			let mut globe = globe.borrow_mut();
			let zoom = 2;
			let n = 2_i32.pow(zoom);

			for y in 0..n {
				for x in 0..n {
					globe.queue_tile(x, y, zoom as i32);
					globe.update().await;
				}
			}
			Ok(true.into())
		}
	});

	// Run every animation frame
	env.animate_loop = web::request_animation_loop({
		let renderer = env.renderer.clone();
		let scene = env.scene.clone();
		let input = env.inputs.clone();
		let mut last_frame_time = now() / 1000.0;

		// Loop
		move || {
			let now = now() / 1000.0;
			let dt = now - last_frame_time;
			last_frame_time = now;


			if let Ok(mut scene) = scene.try_borrow_mut() {
				if let Ok(inputs) = input.try_borrow() {
					scene.tick(dt, &*inputs);
					if let Ok(mut renderer) = renderer.try_borrow_mut() {
						let (w, h) = renderer.size();
						scene.camera_mut().resize(w as f32, h as f32);
						renderer.draw(&scene);
					} else {
						log("Failed to borrow renderer");
					}
				} else {
					log("Failed borrow input state");
				}
			} else {
				log("Failed to borrow scene");
			}
		}
	});

	env
}
