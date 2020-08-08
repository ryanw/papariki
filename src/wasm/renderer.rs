use crate::camera::Camera;
use crate::mesh::Mesh;
use crate::scene::Scene;
use crate::wasm::{self, web, GlMesh, GlPolyline};
use nalgebra as na;
use std::cell::RefCell;
use std::f32::consts::PI;
use std::rc::Rc;
use wasm_bindgen::JsCast;
use web_sys::{
	HtmlCanvasElement, HtmlElement, MouseEvent, WebGlProgram, WebGlRenderingContext, WebGlShader, WheelEvent,
};

fn map_range(val: f32, min0: f32, max0: f32, min1: f32, max1: f32) -> f32 {
	(val - min0) * (max1 - min1) / (max0 - min0) + min1
}

const MESH_PROGRAM: usize = 0;
const POLYLINE_PROGRAM: usize = 1;

static VERTEX_GLSL: &'static str = "
	uniform mat4 view_proj;
	uniform mat4 model;
	attribute vec3 position;
	varying vec4 color;

	void main(void) {
		mat4 mvp = view_proj * model;
		gl_Position = mvp * vec4(position, 1.0);
		color = (vec4(position, 1.0) * 0.5 + 0.5) * (2.0 - (gl_Position.z / 1.5));
	}
";

static FRAGMENT_GLSL: &'static str = "
	precision mediump float;

	varying vec4 color;

	void main(void) {
		gl_FragColor = color;
	}
";


static POLYLINE_VERTEX_GLSL: &'static str = "
	uniform mat4 view_proj;
	uniform mat4 model;
	uniform float zoom;
	attribute vec3 position;
	attribute vec4 normal;
	varying vec4 color;

	void main(void) {
		float thickness = zoom;
		vec3 delta = normal.xyz * thickness;
		mat4 mvp = view_proj * model;
		gl_Position = mvp * vec4(position + delta, 1.0);
		color = vec4(normal.w);
	}
";

static POLYLINE_FRAGMENT_GLSL: &'static str = "
	precision mediump float;
	float map(float value, float min1, float max1, float min2, float max2) {
		return min2 + (value - min1) * (max2 - min2) / (max1 - min1);
	}

	varying vec4 color;

	void main(void) {
		float dist = smoothstep(0.2, 0.2, abs(color.r * 2.0 - 1.0) - 0.4);
		if (dist > 0.5) {
			gl_FragColor = vec4(0.3, 0.4, 0.3, 1.0);
		}
		else {
			gl_FragColor = vec4(0.1, 0.1, 0.2, 1.0);
		}
	}
";

fn rad_to_deg(rad: f32) -> f32 {
	((rad * (180.0 / PI) + 180.0) % 360.0) - 180.0
}

pub struct WebGlRenderer {
	width: i32,
	height: i32,
	last_frame_at: f64,
	element: Option<HtmlCanvasElement>,
	meshes: Vec<GlMesh>,
	lines: Vec<GlPolyline>,
	programs: Vec<WebGlProgram>,
	context: Option<WebGlRenderingContext>,
	globe_rotation: na::Vector3<f32>,
	zoom: f32,
	prev_mouse_position: Option<(i32, i32)>,
	prev_wheel_position: Option<(f32, f32)>,
}

impl WebGlRenderer {
	pub fn new(width: i32, height: i32) -> Self {
		Self {
			width,
			height,
			last_frame_at: wasm::now(),
			element: None,
			meshes: vec![],
			lines: vec![],
			programs: vec![],
			context: None,
			zoom: 1.0,
			//globe_transform: na::Matrix4::from_euler_angles(0.1, 0.0, 0.41),
			globe_rotation: na::Vector3::default(),
			prev_mouse_position: Some((0, 0)),
			prev_wheel_position: Some((0.0, 0.0)),
		}
	}

	pub fn attach(&mut self, container: &HtmlElement) {
		wasm::log("Attaching WebGlRenderer");
		self.attach_to_element(container);
		self.initialize_webgl();
		self.compile_mesh_program();
		self.compile_polyline_program();
	}

	pub fn size(&self) -> (i32, i32) {
		(self.width, self.height)
	}

	fn attach_to_element(&mut self, container: &HtmlElement) {
		let window = web_sys::window().unwrap();
		let document = window.document().unwrap();
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
			let ctx = el
				.get_context("webgl")
				.unwrap()
				.unwrap()
				.dyn_into::<WebGlRenderingContext>()
				.unwrap();
			self.context = Some(ctx);
		} else {
			wasm::log("Unable to obtain WebGlRenderingContext");
			return;
		};
	}

	fn compile_mesh_program(&mut self) {
		let vertex_shader = self.create_vertex_shader(VERTEX_GLSL).unwrap();
		let fragment_shader = self.create_fragment_shader(FRAGMENT_GLSL).unwrap();

		if let Some(gl) = &self.context {
			// Enable 32bit index buffers
			gl.get_extension("OES_element_index_uint").unwrap();
			gl.enable(WebGlRenderingContext::DEPTH_TEST);

			gl.viewport(0, 0, self.width, self.height);
			let program = gl.create_program().unwrap();
			gl.attach_shader(&program, &vertex_shader);
			gl.attach_shader(&program, &fragment_shader);
			gl.link_program(&program);
			gl.use_program(Some(&program));

			gl.clear_color(0.0, 0.0, 0.0, 0.0);
			gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

			self.programs.push(program);
		}
	}

	fn compile_polyline_program(&mut self) {
		let vertex_shader = self.create_vertex_shader(POLYLINE_VERTEX_GLSL).unwrap();
		let fragment_shader = self.create_fragment_shader(POLYLINE_FRAGMENT_GLSL).unwrap();

		if let Some(gl) = &self.context {
			// Enable 32bit index buffers
			gl.get_extension("OES_element_index_uint").unwrap();
			gl.enable(WebGlRenderingContext::DEPTH_TEST);

			gl.viewport(0, 0, self.width, self.height);
			let program = gl.create_program().unwrap();
			gl.attach_shader(&program, &vertex_shader);
			gl.attach_shader(&program, &fragment_shader);
			gl.link_program(&program);
			gl.use_program(Some(&program));

			gl.clear_color(0.0, 0.0, 0.0, 0.0);
			gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

			self.programs.push(program);
		}
	}

	pub fn add_mesh(&mut self, mesh: &Mesh) -> usize {
		wasm::log("Adding mesh");
		let vertices = mesh.vertices_as_vec();
		let triangles = mesh.triangles_as_vec();
		if let Some(gl) = &self.context {
			let mut gl_mesh = GlMesh::new(vertices, triangles);
			gl_mesh.upload(gl);
			self.meshes.push(gl_mesh);
			self.meshes.len()
		} else {
			panic!("Renderer isn't attached");
		}
	}

	pub fn draw(&mut self, scene: &Scene) {
		let now = wasm::now();
		let dt = now - self.last_frame_at;
		//self.draw_meshes(scene);
		self.draw_polylines(scene, dt);
		self.last_frame_at = now;
	}

	fn draw_polylines(&mut self, scene: &Scene, dt: f64) {
		let program = &self.programs[POLYLINE_PROGRAM];

		if let Some(gl) = &self.context {
			gl.use_program(Some(program));
			let position_attrib = gl.get_attrib_location(&program, "position") as u32;
			let normal_attrib = gl.get_attrib_location(&program, "normal") as u32;
			gl.enable_vertex_attrib_array(position_attrib);
			gl.enable_vertex_attrib_array(normal_attrib);

			gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

			// Camera
			let camera = scene.camera();
			let vp = camera.projection() * camera.view();
			let vp_uniform = gl.get_uniform_location(program, "view_proj");
			gl.uniform_matrix4fv_with_f32_array(vp_uniform.as_ref(), false, vp.as_slice());

			// Zoom
			let zoom_uniform = gl.get_uniform_location(program, "zoom");
			let zoom = map_range(scene.zoom.powf(2.0), 0.0, 1.0, 1.0, 0.05);
			//let zoom = (self.last_frame_at / 1000.0).sin() as f32 + 1.0;
			gl.uniform1f(zoom_uniform.as_ref(), zoom * 0.01);

			for (i, item) in scene.items().iter().filter(|i| i.tile.is_some()).enumerate() {
				if i > self.lines.len() {
					panic!("We lost a line");
				}
				if i == self.lines.len() {
					let mut line = GlPolyline::from(&item.polylines());
					line.upload(gl);
					self.lines.push(line);
				}
				let line = &self.lines[i];
				let transform = &item.transform;

				if line.count == 0 {
					continue;
				}
				line.bind(gl);

				let model_uniform = gl.get_uniform_location(program, "model");
				gl.uniform_matrix4fv_with_f32_array(model_uniform.as_ref(), false, transform.as_slice());

				let position_size = 4 * 3;
				let normal_size = 4 * 4;
				let vertex_size = position_size + normal_size;
				let position_attrib = gl.get_attrib_location(program, "position") as u32;
				gl.vertex_attrib_pointer_with_i32(position_attrib, 3, WebGlRenderingContext::FLOAT, false, vertex_size, 0);
				let normal_attrib = gl.get_attrib_location(program, "normal") as u32;
				gl.vertex_attrib_pointer_with_i32(normal_attrib, 4, WebGlRenderingContext::FLOAT, false, vertex_size, position_size);

				gl.draw_elements_with_i32(
					WebGlRenderingContext::TRIANGLES,
					line.count as i32,
					WebGlRenderingContext::UNSIGNED_INT,
					0,
				);
			}

			gl.disable_vertex_attrib_array(position_attrib);
			gl.disable_vertex_attrib_array(normal_attrib);
		}
	}

	fn draw_meshes(&mut self, scene: &Scene) {
		let program = &self.programs[MESH_PROGRAM];

		if let Some(gl) = &self.context {
			gl.use_program(Some(program));
			let position_attrib = gl.get_attrib_location(&program, "position") as u32;
			gl.enable_vertex_attrib_array(position_attrib);

			gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

			// Camera
			let camera = scene.camera();
			let vp = camera.projection() * camera.view();
			let vp_uniform = gl.get_uniform_location(program, "view_proj");
			gl.uniform_matrix4fv_with_f32_array(vp_uniform.as_ref(), false, vp.as_slice());

			for (i, item) in scene.items().iter().enumerate() {
				if i > self.meshes.len() {
					panic!("We lost a mesh");
				}
				if i == self.meshes.len() {
					let mut mesh = GlMesh::from(&item.mesh);
					mesh.upload(gl);
					self.meshes.push(mesh);
				}
				let mesh = &self.meshes[i];
				let transform = &item.transform;

				if mesh.count == 0 {
					continue;
				}
				mesh.bind(gl);

				let model_uniform = gl.get_uniform_location(program, "model");
				gl.uniform_matrix4fv_with_f32_array(model_uniform.as_ref(), false, transform.as_slice());

				let position_attrib = gl.get_attrib_location(program, "position") as u32;
				gl.vertex_attrib_pointer_with_f64(position_attrib, 3, WebGlRenderingContext::FLOAT, false, 0, 0.0);

				gl.draw_elements_with_i32(
					WebGlRenderingContext::TRIANGLES,
					mesh.count as i32,
					WebGlRenderingContext::UNSIGNED_INT,
					0,
				);
			}

			gl.disable_vertex_attrib_array(position_attrib);
		}
	}

	fn create_shader(&self, kind: u32, glsl: &str) -> Option<WebGlShader> {
		if let Some(gl) = &self.context {
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
