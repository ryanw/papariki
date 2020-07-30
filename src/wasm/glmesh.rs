use js_sys::{Float32Array, Uint32Array};
use nalgebra as na;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

pub struct GlMesh {
	pub(super) index_buffer: WebGlBuffer,
	pub(super) vertex_buffer: WebGlBuffer,
	pub(super) transform: na::Matrix4<f32>,
	pub(super) count: u32,
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
