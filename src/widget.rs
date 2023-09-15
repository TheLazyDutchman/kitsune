use crate::{
	context::Context,
	render::{Render, RenderedMesh, Vertex},
	text::Font,
};

pub trait Widget {
	type Renderable: Render;

	fn get_renderable(&mut self, context: &mut Context<WidgetContext>) -> Self::Renderable;
}

pub struct WidgetContext<'a> {
	font: &'a Font,
	device: &'a wgpu::Device,
	queue: &'a wgpu::Queue,
}

impl<'a> WidgetContext<'a> {
	pub fn new(device: &'a wgpu::Device, queue: &'a wgpu::Queue, font: &'a Font) -> Self {
		Self {
			device,
			font,
			queue,
		}
	}
}

impl Widget for char {
	type Renderable = RenderedMesh;

	fn get_renderable(
		&mut self,
		context: &mut crate::context::Context<crate::widget::WidgetContext>,
	) -> Self::Renderable {
		let vertices = [
			Vertex::new([-1.0, 1.0], [0.0, 0.0]),
			Vertex::new([-1.0, -1.0], [0.0, 1.0]),
			Vertex::new([1.0, 1.0], [1.0, 0.0]),
			Vertex::new([1.0, -1.0], [1.0, 1.0]),
		];

		let indices = [0, 1, 2, 1, 3, 2];

		let bind_group = context
			.font
			.rasterize(*self, context.device, context.queue);

		RenderedMesh::new(context.device, &vertices, &indices, bind_group)
	}
}

use paste::paste;
macro_rules! tuple_impl {
    ($($name:ident),*) => {
        impl<$($name: Widget),*> Widget for ($($name),*) {
        	type Renderable = ($($name::Renderable),*);

        	fn get_renderable(&mut self, context: &mut crate::context::Context<crate::widget::WidgetContext>) -> Self::Renderable {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				($(<$name as crate::widget::Widget>::get_renderable([<$name:snake>], context)),*)
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
