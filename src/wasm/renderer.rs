use crate::camera::Camera;
use crate::globe::Globe;
use crate::mesh::Mesh;
use crate::wasm;
use js_sys::{ArrayBuffer, Float32Array, Uint16Array, Uint32Array};
use nalgebra as na;
use std::f32::consts::PI;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, HtmlElement, WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlShader};

// FIXME the coordinates are all kinds of messed up. X and Z are swapped.
static VERTEX_GLSL: &'static str = "
	uniform mat4 view_proj;
	uniform mat4 model;
	attribute vec3 position;
	varying vec4 color;

	void main(void) {
		mat4 mvp = view_proj * model;
		gl_Position = mvp * vec4(position, 1.0);
		color = (vec4(position, 1.0) * 0.5 + 0.5) * (2.0 - (gl_Position.z / 2.0));
	}
";

static FRAGMENT_GLSL: &'static str = "
	precision mediump float;

	varying vec4 color;

	void main(void) {
		gl_FragColor = color;
	}
";

struct GlMesh {
	index_buffer: WebGlBuffer,
	vertex_buffer: WebGlBuffer,
	transform: na::Matrix4<f32>,
	count: u32,
}

impl GlMesh {
	pub fn new(gl: &WebGlRenderingContext) -> Self {
		Self {
			vertex_buffer: gl.create_buffer().unwrap(),
			index_buffer: gl.create_buffer().unwrap(),
			transform: na::Matrix4::identity(),
			count: 0,
		}
	}
	pub fn upload_vertices(&mut self, gl: &WebGlRenderingContext, vertices: &[f32]) {
		gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));
		gl.buffer_data_with_opt_array_buffer(
			WebGlRenderingContext::ARRAY_BUFFER,
			Some(&Float32Array::from(vertices).buffer()),
			WebGlRenderingContext::STATIC_DRAW,
		);
	}

	pub fn upload_indices(&mut self, gl: &WebGlRenderingContext, indices: &[u32]) {
		self.count = indices.len() as u32;

		gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
		gl.buffer_data_with_opt_array_buffer(
			WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
			Some(&Uint32Array::from(indices).buffer()),
			WebGlRenderingContext::STATIC_DRAW,
		);
	}
}

pub struct WebGlRenderer {
	width: i32,
	height: i32,
	last_frame_at: f64,
	element: Option<HtmlCanvasElement>,
	meshes: Vec<GlMesh>,
	program: Option<WebGlProgram>,
	context: Option<WebGlRenderingContext>,
	globe_transform: na::Matrix4<f32>,
	camera: Camera,
}

impl WebGlRenderer {
	pub fn new(width: i32, height: i32) -> Self {
		Self {
			width,
			height,
			last_frame_at: wasm::now(),
			element: None,
			meshes: vec![],
			program: None,
			context: None,
			globe_transform: na::Matrix4::identity(),
			camera: Camera::new(width as f32, height as f32),
		}
	}

	pub fn attach(&mut self, container: &HtmlElement) {
		wasm::log("Attaching WebGlRenderer");
		self.attach_to_element(container);
		self.initialize_webgl();
	}

	pub fn tick(&mut self) {
		let now = wasm::now() / 1000.0;
		let dt = (now - self.last_frame_at) as f32;
		self.globe_transform *= na::Matrix4::from_euler_angles(0.0, -0.5 * dt, 0.0);
		for mesh in &mut self.meshes {
			mesh.transform = self.globe_transform;
		}
		self.last_frame_at = now;
	}

	fn attach_to_element(&mut self, container: &HtmlElement) {
		let document = web_sys::window().unwrap().document().unwrap();
		let el = document
			.create_element("canvas")
			.unwrap()
			.dyn_into::<HtmlCanvasElement>()
			.unwrap();
		el.set_attribute("width", &self.width.to_string()).unwrap();
		el.set_attribute("height", &self.height.to_string()).unwrap();
		container.append_child(&el).unwrap();
		self.element = Some(el);
	}

	fn initialize_webgl(&mut self) {
		if let Some(el) = &self.element {
			let ctx = el.get_context("webgl")
				.unwrap()
				.unwrap()
				.dyn_into::<WebGlRenderingContext>()
				.unwrap();
			self.context = Some(ctx);
		} else {
			wasm::log("Unable to obtain WebGlRenderingContext");
			return;
		};

		let vertex_shader = self.create_vertex_shader(VERTEX_GLSL).unwrap();
		let fragment_shader = self.create_fragment_shader(FRAGMENT_GLSL).unwrap();


		if let Some(gl) = self.webgl_context() {
			// Enable 32bit index buffers
			gl.get_extension("OES_element_index_uint").unwrap();

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
		wasm::log("Adding mesh");
		let vertices = mesh.vertices_as_vec();
		let triangles = mesh.triangles_as_vec();
		if let Some(gl) = &self.webgl_context() {
			let mut gl_mesh = GlMesh::new(gl);

			gl_mesh.upload_vertices(gl, vertices.as_slice());
			gl_mesh.upload_indices(gl, triangles.as_slice());

			self.meshes.push(gl_mesh);
		}
	}

	pub fn update_uniforms(&mut self) {
		if self.program.is_none() {
			return;
		}

		let program = self.program.as_ref().unwrap();
		if let Some(gl) = &self.webgl_context() {
			let vp = self.camera.projection() * self.camera.view();
			let vp_uniform = gl.get_uniform_location(program, "view_proj");
			gl.uniform_matrix4fv_with_f32_array(vp_uniform.as_ref(), false, vp.as_slice());
		}
	}

	pub fn draw(&mut self) {
		self.update_uniforms();

		let program = self.program.as_ref();
		if let Some(gl) = &self.webgl_context() {
			gl.use_program(program);
			gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
		}

		for mesh in &self.meshes {
			if mesh.count == 0 {
				continue;
			}
			if let Some(gl) = &self.webgl_context() {
				let model_uniform = gl.get_uniform_location(program.unwrap(), "model");
				gl.uniform_matrix4fv_with_f32_array(model_uniform.as_ref(), false, mesh.transform.as_slice());

				//wasm::log(&format!("Drawing draw mesh with {} vertices", mesh.count));
				gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&mesh.index_buffer));
				gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&mesh.vertex_buffer));
				// Send vertices to vertex shader "position"
				let position_attrib = gl.get_attrib_location(program.unwrap(), "position") as u32;
				gl.vertex_attrib_pointer_with_f64(position_attrib, 3, WebGlRenderingContext::FLOAT, false, 0, 0.0);

				gl.draw_elements_with_i32(
					WebGlRenderingContext::TRIANGLES,
					mesh.count as i32,
					WebGlRenderingContext::UNSIGNED_INT,
					0,
				);
			} else {
				wasm::log(&format!("Failed to draw mesh with {} vertices", mesh.count));
			}
		}
	}

	pub fn webgl_context(&self) -> Option<&WebGlRenderingContext> {
		self.context.as_ref()
	}

	fn create_shader(&self, kind: u32, glsl: &str) -> Option<WebGlShader> {
		if let Some(gl) = self.webgl_context() {
			let shader = gl.create_shader(kind).unwrap();
			gl.shader_source(&shader, glsl);
			gl.compile_shader(&shader);

			Some(shader)
		} else {
			wasm::log("Failed to create shader");
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
