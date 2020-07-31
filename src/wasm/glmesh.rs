use crate::mesh::Mesh;
use js_sys::{Float32Array, Uint32Array};
use nalgebra as na;
use web_sys::{WebGlBuffer, WebGlRenderingContext};

pub struct GlMesh {
	pub(super) vertices: Vec<f32>,
	pub(super) indices: Vec<u32>,
	pub(super) vertex_buffer: Option<WebGlBuffer>,
	pub(super) index_buffer: Option<WebGlBuffer>,
	pub(super) transform: na::Matrix4<f32>,
	pub(super) count: u32,
}

impl From<&Mesh> for GlMesh {
	fn from(mesh: &Mesh) -> Self {
		Self::new(mesh.vertices_as_vec(), mesh.triangles_as_vec())
	}
}

impl GlMesh {
	pub fn new(vertices: Vec<f32>, indices: Vec<u32>) -> Self {
		Self {
			vertex_buffer: None,
			index_buffer: None,
			vertices,
			indices,
			transform: na::Matrix4::identity(),
			count: 0,
		}
	}

	pub fn uploaded(&self) -> bool {
		self.index_buffer.is_some()
	}

	pub fn upload(&mut self, gl: &WebGlRenderingContext) {
		self.upload_vertices(gl);
		self.upload_indices(gl);
	}

	pub fn bind(&self, gl: &WebGlRenderingContext) {
		if !self.uploaded() {
			panic!("Can't bind gl mesh that wasn't uploaded");
		}
		gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, self.index_buffer.as_ref());
		gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, self.vertex_buffer.as_ref());
	}

	fn upload_vertices(&mut self, gl: &WebGlRenderingContext) {
		let vertex_buffer = gl.create_buffer().unwrap();
		gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
		gl.buffer_data_with_opt_array_buffer(
			WebGlRenderingContext::ARRAY_BUFFER,
			Some(&Float32Array::from(self.vertices.as_slice()).buffer()),
			WebGlRenderingContext::STATIC_DRAW,
		);
		self.vertex_buffer = Some(vertex_buffer);
	}

	fn upload_indices(&mut self, gl: &WebGlRenderingContext) {
		self.count = self.indices.len() as u32;

		let index_buffer = gl.create_buffer().unwrap();
		gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
		gl.buffer_data_with_opt_array_buffer(
			WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
			Some(&Uint32Array::from(self.indices.as_slice()).buffer()),
			WebGlRenderingContext::STATIC_DRAW,
		);

		self.index_buffer = Some(index_buffer);
	}
}
