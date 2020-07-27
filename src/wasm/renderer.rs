use crate::camera::Camera;
use crate::globe::Globe;
use crate::wasm;
use crate::geometry::Mesh;
use js_sys::{ArrayBuffer, Float32Array, Uint16Array, Uint32Array};
use nalgebra as na;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};
use std::f32::consts::PI;

// FIXME the coordinates are all kinds of messed up. X and Z are swapped.
static VERTEX_GLSL: &'static str = "
	uniform mat4 mvp;
	attribute vec3 position;

	void main(void) {
		gl_Position = mvp * vec4(position, 1.0);
	}
";

static FRAGMENT_GLSL: &'static str = "
	void main(void) {
		gl_FragColor = vec4(1.0, 0.0, 1.0, 1.0);
	}
";

#[wasm_bindgen]
pub struct WebGlRenderer {
	width: i32,
	height: i32,
	last_frame_at: f64,
	element: Option<HtmlCanvasElement>,
	element_counts: Vec<i32>,
	index_buffers: Vec<WebGlBuffer>,
	vertex_buffers: Vec<WebGlBuffer>,
	program: Option<WebGlProgram>,

	model: na::Matrix4<f32>,
	camera: Camera,
}

#[wasm_bindgen]
impl WebGlRenderer {
	pub fn attach(&mut self, container: &HtmlElement) {
		wasm::log("Attaching WebGlRenderer");
		self.attach_to_element(container);
		self.initialize_webgl();
	}

	pub fn tick(&mut self) {
		let now = wasm::now() / 1000.0;
		let dt = (now - self.last_frame_at) as f32;
		self.model *= na::Matrix4::from_euler_angles(0.0, -0.5 * dt, 0.0);
		self.last_frame_at = now;
	}
}

impl WebGlRenderer {
	pub fn new(width: i32, height: i32) -> Self {
		Self {
			width,
			height,
			last_frame_at: wasm::now(),
			element: None,
			element_counts: vec![],
			index_buffers: vec![],
			vertex_buffers: vec![],
			program: None,

			model: na::Matrix4::from_euler_angles(0.0, PI / 2.0, 0.0),
			camera: Camera::new(width as f32, height as f32),
		}
	}

	fn attach_to_element(&mut self, container: &HtmlElement) {
		let document = web_sys::window().unwrap().document().unwrap();
		let el = document
			.create_element("canvas")
			.unwrap()
			.dyn_into::<HtmlCanvasElement>()
			.unwrap();
		el.set_attribute("width", &self.width.to_string());
		el.set_attribute("height", &self.height.to_string());
		container.append_child(&el);
		self.element = Some(el);
	}

	fn initialize_webgl(&mut self) {
		let vertex_shader = self.create_vertex_shader(VERTEX_GLSL).unwrap();
		let fragment_shader = self.create_fragment_shader(FRAGMENT_GLSL).unwrap();

		if let Some(gl) = self.webgl_context() {
			// Enable 32bit index buffers
			gl.get_extension("OES_element_index_uint");

			gl.viewport(0, 0, self.width, self.height);
			let program = gl.create_program().unwrap();
			gl.attach_shader(&program, &vertex_shader);
			gl.attach_shader(&program, &fragment_shader);
			gl.link_program(&program);
			gl.use_program(Some(&program));

			gl.clear_color(0.0, 0.0, 0.0, 0.0);
			gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

			// Send vertices to vertex shader "position"
			let position_attrib = gl.get_attrib_location(&program, "position") as u32;
			gl.enable_vertex_attrib_array(position_attrib);

			self.program = Some(program);
		}
	}

	pub fn add_mesh(&mut self, mesh: Mesh) {
		self.add_vertices(mesh.vertices_as_vec());
		self.add_triangles(mesh.triangles_as_vec());
	}

	pub fn model(&self) -> &na::Matrix4<f32> {
		&self.model
	}

	pub fn model_mut(&mut self) -> &mut na::Matrix4<f32> {
		&mut self.model
	}

	pub fn update_uniforms(&mut self) {
		if self.program.is_none() {
			return;
		}

		let program = self.program.as_ref().unwrap();
		if let Some(gl) = &self.webgl_context() {
			let mvp = self.camera.projection() * self.camera.view() * self.model;
			let mvp_uniform = gl.get_uniform_location(program, "mvp");
			gl.uniform_matrix4fv_with_f32_array(mvp_uniform.as_ref(), false, mvp.as_slice());
		}
	}

	pub fn add_vertices(&mut self, vertices: Vec<f32>) {
		let program = self.program.as_ref().unwrap();
		if let Some(gl) = &self.webgl_context() {
			// Create and populate vertex buffer
			let vertex_buffer = gl.create_buffer().unwrap();
			gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
			gl.buffer_data_with_opt_array_buffer(
				WebGlRenderingContext::ARRAY_BUFFER,
				Some(&Float32Array::from(&vertices[..]).buffer()),
				WebGlRenderingContext::STATIC_DRAW,
			);


			self.vertex_buffers.push(vertex_buffer);
		}
	}

	pub fn add_triangles(&mut self, triangles: Vec<u32>) {
		if let Some(gl) = &self.webgl_context() {
			self.element_counts.push(triangles.len() as i32);

			// Create and populate index buffer
			let index_buffer = gl.create_buffer().unwrap();
			gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
			gl.buffer_data_with_opt_array_buffer(
				WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
				Some(&Uint32Array::from(&triangles[..]).buffer()),
				WebGlRenderingContext::STATIC_DRAW,
			);
			self.index_buffers.push(index_buffer);
		}
	}

	pub fn draw(&mut self) {
		self.update_uniforms();

		let program = self.program.as_ref();
		if let Some(gl) = &self.webgl_context() {
			gl.use_program(program);
			gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
		}

		let count = self.element_counts.len();
		for i in 0..count {
			let index_buffer = self.index_buffers.get(i);
			let vertex_buffer = self.vertex_buffers.get(i);
			let element_count = *self.element_counts.get(i).unwrap_or(&0);
			if element_count == 0 {
				continue;
			}
			if let Some(gl) = &self.webgl_context() {
				//wasm::log(&format!("Drawing draw mesh {} with {} vertices", i, element_count));
				gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, index_buffer);
				gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, vertex_buffer);
				// Send vertices to vertex shader "position"
				let position_attrib = gl.get_attrib_location(program.unwrap(), "position") as u32;
				gl.vertex_attrib_pointer_with_f64(position_attrib, 3, WebGlRenderingContext::FLOAT, false, 0, 0.0);


				gl.draw_elements_with_i32(
					WebGlRenderingContext::TRIANGLES,
					element_count,
					WebGlRenderingContext::UNSIGNED_INT,
					0,
				);
			}
			else {
				wasm::log(&format!("Failed to draw mesh {} with {} vertices", i, element_count));
			}
		}
	}

	pub fn webgl_context(&self) -> Option<WebGlRenderingContext> {
		if let Some(el) = &self.element {
			Some(
				el.get_context("webgl")
					.unwrap()
					.unwrap()
					.dyn_into::<WebGlRenderingContext>()
					.unwrap(),
			)
		} else {
			None
		}
	}

	fn create_shader(&self, kind: u32, glsl: &str) -> Option<WebGlShader> {
		if let Some(gl) = self.webgl_context() {
			let shader = gl.create_shader(kind).unwrap();
			gl.shader_source(&shader, glsl);
			gl.compile_shader(&shader);

			Some(shader)
		} else {
			None
		}
	}

	fn create_vertex_shader(&self, glsl: &str) -> Option<WebGlShader> {
		self.create_shader(WebGlRenderingContext::VERTEX_SHADER, glsl)
	}

	fn create_fragment_shader(&self, glsl: &str) -> Option<WebGlShader> {
		self.create_shader(WebGlRenderingContext::FRAGMENT_SHADER, glsl)
	}
}
