use wgpu::util::DeviceExt;

use crate::{context::Context, view::GlobalPosition};

pub trait Render {
	fn render<'a, 'b>(&'a self, context: &mut Context<RenderContext<'b>>)
	where
		'a: 'b;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
	position: GlobalPosition,
	uv: [f32; 2],
}

impl Vertex {
	const LAYOUT: [wgpu::VertexAttribute; 2] =
		wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2];

	pub fn layout() -> wgpu::VertexBufferLayout<'static> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Self>() as u64,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &Self::LAYOUT,
		}
	}

	pub fn new(position: GlobalPosition, uv: [f32; 2]) -> Self {
		Self { position, uv }
	}
}

pub struct RenderContext<'a> {
	pass: wgpu::RenderPass<'a>,
}

impl<'a> RenderContext<'a> {
	pub fn new(pass: wgpu::RenderPass<'a>) -> Self {
		Self { pass }
	}
}

pub struct RenderedMesh {
	vertices: wgpu::Buffer,
	indices: wgpu::Buffer,
	num_indices: u32,
	bind_group: wgpu::BindGroup,
}

impl RenderedMesh {
	pub fn new(
		device: &wgpu::Device,
		vertices: &[Vertex],
		indices: &[u16],
		bind_group: wgpu::BindGroup,
	) -> Self {
		let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Vertex Buffer"),
			contents: bytemuck::cast_slice(vertices),
			usage: wgpu::BufferUsages::VERTEX,
		});

		let num_indices = indices.len() as u32;

		let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
			label: Some("Index Buffer"),
			contents: bytemuck::cast_slice(indices),
			usage: wgpu::BufferUsages::INDEX,
		});

		Self {
			vertices,
			indices,
			num_indices,
			bind_group,
		}
	}
}

mod impls {
	use paste::paste;

	use super::*;

	impl Render for RenderedMesh {
		fn render<'a, 'b>(&'a self, context: &mut Context<RenderContext<'b>>)
		where
			'a: 'b,
		{
			context
				.pass
				.set_bind_group(0, &self.bind_group, &[]);
			context
				.pass
				.set_vertex_buffer(0, self.vertices.slice(..));
			context
				.pass
				.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint16);
			context
				.pass
				.draw_indexed(0..self.num_indices, 0, 0..1);
		}
	}

	macro_rules! tuple_impl {
    ($($name:ident),*) => {
        impl<$($name: Render),*> Render for ($($name),*) {
        	fn render<'a, 'b>(&'a self, context: &mut crate::context::Context<crate::render::RenderContext<'b>>) where 'a: 'b{
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				$(
    				    <$name as crate::render::Render>::render([<$name:snake>], context);
    				)*
    			}
    		}
        }
    };
}

	macro_rules! tuples_impl {
	($(($($name:ident),*)),*) => {
	    $(
	        tuple_impl!($($name),*);
	    )*
	};
}

	tuples_impl!(
		(A, B),
		(A, B, C),
		(A, B, C, D),
		(A, B, C, D, E),
		(A, B, C, D, E, F),
		(A, B, C, D, E, F, G),
		(A, B, C, D, E, F, G, H)
	);
}
