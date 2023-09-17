use crate::{
	context::Context,
	render::{Render, RenderedMesh, Vertex},
	text::Font,
	view::View,
};

pub trait Widget {
	type Renderable: Render;

	fn get_renderable(
		&mut self,
		context: &mut Context<WidgetContext>,
		view: View,
	) -> Self::Renderable;
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

pub struct Row<T> {
	values: Vec<T>,
}

impl<T> Row<T> {
	pub fn new(values: Vec<T>) -> Self {
		Self { values }
	}
}

impl<T> std::ops::Deref for Row<T> {
	type Target = Vec<T>;

	fn deref(&self) -> &Self::Target {
		&self.values
	}
}

impl<T> std::ops::DerefMut for Row<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.values
	}
}

pub struct Column<T> {
	values: Vec<T>,
}

impl<T> Column<T> {
	pub fn new(values: Vec<T>) -> Self {
		Self { values }
	}
}

impl<T> std::ops::Deref for Column<T> {
	type Target = Vec<T>;

	fn deref(&self) -> &Self::Target {
		&self.values
	}
}

impl<T> std::ops::DerefMut for Column<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.values
	}
}

mod impls {
	use paste::paste;

	use super::*;
	use crate::view::VirtualPosition;

	#[cfg(feature = "text")]
	impl Widget for char {
		type Renderable = Option<RenderedMesh>;

		fn get_renderable(
			&mut self,
			context: &mut crate::context::Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			let Some(bind_group) = context
				.font
				.rasterize(*self, context.device, context.queue)
			else {
				return None;
			};

			let vertices = [
				Vertex::new(view.globalize(VirtualPosition::new(0.0, 0.0)), [0.0, 0.0]),
				Vertex::new(view.globalize(VirtualPosition::new(0.0, 1.0)), [0.0, 1.0]),
				Vertex::new(view.globalize(VirtualPosition::new(1.0, 0.0)), [1.0, 0.0]),
				Vertex::new(view.globalize(VirtualPosition::new(1.0, 1.0)), [1.0, 1.0]),
			];

			let indices = [0, 1, 2, 1, 3, 2];

			Some(RenderedMesh::new(
				context.device,
				&vertices,
				&indices,
				bind_group,
			))
		}
	}

	#[cfg(feature = "text")]
	impl Widget for String {
		type Renderable = Vec<RenderedMesh>;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			Row::new(self.chars().collect())
				.get_renderable(context, view)
				.into_iter()
				.filter_map(|x| x)
				.collect()
		}
	}

	impl<T> Widget for Row<T>
	where
		T: Widget,
	{
		type Renderable = Vec<T::Renderable>;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			let views = view.split_row(self.values.len() as u32);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}
	}

	impl<T> Widget for Column<T>
	where
		T: Widget,
	{
		type Renderable = Vec<T::Renderable>;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			let views = view.split_column(self.values.len() as u32);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}
	}

	macro_rules! tuple_impl {
    ($($name:ident),*) => {
        impl<$($name: Widget),*> Widget for ($($name),*) {
        	type Renderable = ($($name::Renderable),*);

        	fn get_renderable(&mut self, context: &mut crate::context::Context<WidgetContext>, view: crate::view::View) -> Self::Renderable {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				($(<$name as Widget>::get_renderable([<$name:snake>], context, view.clone())),*)
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
