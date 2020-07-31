use crate::wasm;
use crate::wasm::web;
use crate::input::UserInputs;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{HtmlElement, MouseEvent, WheelEvent};

#[derive(Clone, Debug, Default)]
pub struct InputState {
	mouse_down: bool,
	mouse_position: (i32, i32),
	wheel_position: (f32, f32),
}

impl InputState {
	pub fn new() -> Self {
		Default::default()
	}
}

#[derive(Clone, Debug, Default)]
pub struct HtmlInputs {
	state: Rc<RefCell<InputState>>,
}

impl UserInputs for HtmlInputs {
	fn is_mouse_down(&self) -> bool {
		self.state.borrow().mouse_down
	}

	fn mouse_position(&self) -> (i32, i32) {
		self.state.borrow().mouse_position.clone()
	}

	fn wheel_position(&self) -> (f32, f32) {
		self.state.borrow().wheel_position.clone()
	}
}

impl HtmlInputs {
	pub fn attach(&mut self, el: &HtmlElement) {
		wasm::log("Attaching Input Handlers");

		let window = web_sys::window().unwrap();

		let state = self.state.clone();
		web::add_event_listener(&el, "mousedown", move |e: MouseEvent| {
			if let Ok(mut state) = state.try_borrow_mut() {
				let x = e.client_x();
				let y = e.client_y();
				state.mouse_position = (x, y);
				state.mouse_down = true;
			}
		})
		.unwrap();

		let state = self.state.clone();
		web::add_event_listener(&window, "mouseup", move |e: MouseEvent| {
			if let Ok(mut state) = state.try_borrow_mut() {
				let x = e.client_x();
				let y = e.client_y();
				state.mouse_position = (x, y);
				state.mouse_down = false;
			}
		})
		.unwrap();

		let state = self.state.clone();
		web::add_event_listener(&window, "mousemove", move |e: MouseEvent| {
			if let Ok(mut state) = state.try_borrow_mut() {
				if !state.mouse_down {
					return;
				}
				let x = e.client_x();
				let y = e.client_y();
				state.mouse_position = (x, y);
			}
		})
		.unwrap();

		let state = self.state.clone();
		web::add_event_listener(&window, "wheel", move |e: WheelEvent| {
			if let Ok(mut state) = state.try_borrow_mut() {
				let mut x = e.delta_x() as f32;
				let mut y = e.delta_y() as f32;
				// FIXME adjustment for chrome vs firefox
				if x > 50.0 {
					x -= 50.0;
				}
				if x < -50.0 {
					x += 50.0;
				}
				if y > 50.0 {
					y -= 50.0;
				}
				if y < -50.0 {
					y += 50.0;
				}

				state.wheel_position.0 += x;
				state.wheel_position.1 += y;
			}
		})
		.unwrap();
	}
}
