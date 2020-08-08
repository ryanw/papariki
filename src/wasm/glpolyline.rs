use crate::polyline::Polyline;
use js_sys::{Float32Array, Uint32Array};
use nalgebra as na;
use web_sys::{WebGlBuffer, WebGlRenderingContext};
use std::f32::consts::PI;
use crate::wasm;

pub struct GlPolyline {
	pub(super) lines: Vec<Polyline>,
	pub(super) vertex_buffer: Option<WebGlBuffer>,
	pub(super) index_buffer: Option<WebGlBuffer>,
	pub(super) transform: na::Matrix4<f32>,
	pub(super) count: u32,
}

impl From<&Vec<Polyline>> for GlPolyline {
	fn from(lines: &Vec<Polyline>) -> Self {
		Self::new(lines.clone())
	}
}

impl GlPolyline {
	pub fn new(lines: Vec<Polyline>) -> Self {
		Self {
			lines,
			vertex_buffer: None,
			index_buffer: None,
			transform: na::Matrix4::identity(),
			count: 0,
		}
	}

	pub fn uploaded(&self) -> bool {
		self.index_buffer.is_some()
	}

	pub fn bind(&self, gl: &WebGlRenderingContext) {
		if !self.uploaded() {
			panic!("Can't bind gl mesh that wasn't uploaded");
		}
		gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, self.index_buffer.as_ref());
		gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, self.vertex_buffer.as_ref());
	}

	pub fn upload(&mut self, gl: &WebGlRenderingContext) {
		let mut vertices = vec![];
		let mut indices = vec![];


		for line in &self.lines {
			for ring in &line.rings {
				for p in ring.points.windows(2) {
					let p0 = p[0];
					let p1 = p[1];

					let normal0 = p0.coords.cross(&p1.coords).normalize();
					let normal1 = p1.coords.cross(&p0.coords).normalize();

					// Add the quad
					let offset = vertices.len() as u32 / 7;

					// Position
					vertices.push(p0.x); // X
					vertices.push(p0.y); // Y
					vertices.push(p0.z); // Z

					// Color
					vertices.push(normal0.x); // R
					vertices.push(normal0.y); // G
					vertices.push(normal0.z); // B
					vertices.push(1.0); // A

					// Position
					vertices.push(p1.x); // X
					vertices.push(p1.y); // Y
					vertices.push(p1.z); // Z

					// Color
					vertices.push(normal0.x); // R
					vertices.push(normal0.y); // G
					vertices.push(normal0.z); // B
					vertices.push(1.0); // A

					// Position
					vertices.push(p0.x); // X
					vertices.push(p0.y); // Y
					vertices.push(p0.z); // Z

					// Color
					vertices.push(normal1.x); // R
					vertices.push(normal1.y); // G
					vertices.push(normal1.z); // B
					vertices.push(0.0); // A

					// Position
					vertices.push(p1.x); // X
					vertices.push(p1.y); // Y
					vertices.push(p1.z); // Z

					// Color
					vertices.push(normal1.x); // R
					vertices.push(normal1.y); // G
					vertices.push(normal1.z); // B
					vertices.push(0.0); // A

					indices.push(offset + 0);
					indices.push(offset + 1);
					indices.push(offset + 2);

					indices.push(offset + 1);
					indices.push(offset + 2);
					indices.push(offset + 3);
				}
			}
		}
		self.count = indices.len() as u32;

		// Upload verts
		let vertex_buffer = gl.create_buffer().unwrap();
		gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
		gl.buffer_data_with_opt_array_buffer(
			WebGlRenderingContext::ARRAY_BUFFER,
			Some(&Float32Array::from(vertices.as_slice()).buffer()),
			WebGlRenderingContext::STATIC_DRAW,
		);

		self.vertex_buffer = Some(vertex_buffer);



		// Upload indieces
		let index_buffer = gl.create_buffer().unwrap();
		gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
		gl.buffer_data_with_opt_array_buffer(
			WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
			Some(&Uint32Array::from(indices.as_slice()).buffer()),
			WebGlRenderingContext::STATIC_DRAW,
		);

		self.index_buffer = Some(index_buffer);
	}
}
