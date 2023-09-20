use crate::{
	context::Context,
	render::{Render, RenderedMesh},
	text::Font,
	view::{SizeHint, View},
};

pub trait Widget {
	type Renderable: Render;

	fn get_renderable(
		&mut self,
		context: &mut Context<WidgetContext>,
		view: View,
	) -> Self::Renderable;

	fn width_hint(&self) -> SizeHint {
		SizeHint::None
	}

	fn height_hint(&self) -> SizeHint {
		SizeHint::None
	}
}

pub struct WidgetContext<'a> {
	font: &'a Font,
	device: &'a wgpu::Device,
	queue: &'a wgpu::Queue,
	config: &'a wgpu::SurfaceConfiguration,
	sampler: &'a wgpu::Sampler,
	bind_group_layout: &'a wgpu::BindGroupLayout,
}

impl<'a> WidgetContext<'a> {
	pub fn new(
		font: &'a Font,
		device: &'a wgpu::Device,
		queue: &'a wgpu::Queue,
		config: &'a wgpu::SurfaceConfiguration,
		sampler: &'a wgpu::Sampler,
		bind_group_layout: &'a wgpu::BindGroupLayout,
	) -> Self {
		Self {
			font,
			device,
			queue,
			config,
			sampler,
			bind_group_layout,
		}
	}
}

macro_rules! wrapper {
	(
		struct $name:ident<T> {
			$value:ident: $ty:ty
			$(, $field:ident: $field_ty:ty)*
		}
	) => {
		pub struct $name<T> {
			$value: $ty,
			$($field:$field_ty,)*
		}

		impl<T> $name<T> {
			pub fn new($value: $ty, $($field:$field_ty),*) -> Self {
				Self { $value, $($field),* }
			}
		}

		impl<T> std::ops::Deref for $name<T> {
			type Target = $ty;

			fn deref(&self) -> &Self::Target {
				&self.$value
			}
		}

		impl<T> std::ops::DerefMut for $name<T> {
			fn deref_mut(&mut self) -> &mut Self::Target {
				&mut self.$value
			}
		}
	};
}

wrapper! {
	struct Row<T> {
		values: Vec<T>
	}
}
wrapper! {
	struct Column<T> {
		values: Vec<T>
	}
}

wrapper! {
	struct Bordered<T> {
		value: T,
		width: u32
	}
}

mod impls {
	use paste::paste;

	use super::*;
	use crate::texture::Texture;

	#[cfg(feature = "text")]
	impl Widget for char {
		type Renderable = Option<RenderedMesh>;

		fn get_renderable(
			&mut self,
			context: &mut crate::context::Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			let Some(bind_group) = context.font.rasterize(
				*self,
				context.device,
				context.config.format,
				context.queue,
				context.sampler,
				context.bind_group_layout,
			) else {
				return None;
			};

			let view = view.from_size_hints(self.width_hint(), self.height_hint());
			let vertices = view.corners();

			let indices = [0, 1, 2, 1, 3, 2];

			Some(RenderedMesh::new(
				context.device,
				&vertices,
				&indices,
				bind_group,
			))
		}

		fn width_hint(&self) -> SizeHint {
			SizeHint::Physical(10)
		}

		fn height_hint(&self) -> SizeHint {
			SizeHint::Physical(20)
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

		fn width_hint(&self) -> SizeHint {
			SizeHint::Sum(
				self.chars()
					.into_iter()
					.map(|x| x.width_hint())
					.collect(),
			)
		}

		fn height_hint(&self) -> SizeHint {
			SizeHint::Max(
				self.chars()
					.into_iter()
					.map(|x| x.height_hint())
					.collect(),
			)
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
			let view = view.from_size_hints(self.width_hint(), self.height_hint());
			let views = view.split_row(self.values.len() as u32);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}

		fn width_hint(&self) -> SizeHint {
			SizeHint::Sum(
				self.values
					.iter()
					.map(|x| x.width_hint())
					.collect(),
			)
		}

		fn height_hint(&self) -> SizeHint {
			SizeHint::Max(
				self.values
					.iter()
					.map(|x| x.height_hint())
					.collect(),
			)
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
			let view = view.from_size_hints(self.width_hint(), self.height_hint());
			let views = view.split_column(self.values.len() as u32);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}

		fn width_hint(&self) -> SizeHint {
			SizeHint::Max(
				self.values
					.iter()
					.map(|x| x.width_hint())
					.collect(),
			)
		}

		fn height_hint(&self) -> SizeHint {
			SizeHint::Sum(
				self.values
					.iter()
					.map(|x| x.height_hint())
					.collect(),
			)
		}
	}

	impl<T> Widget for Bordered<T>
	where
		T: Widget,
	{
		type Renderable = (RenderedMesh, T::Renderable);

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			let view = view.from_size_hints(self.width_hint(), self.height_hint());
			let (outer, inner) = view.bordered(self.width);

			let size = wgpu::Extent3d {
				width: 10,
				height: 10,
				depth_or_array_layers: 1,
			};

			let mut texture = Texture::new(context.device, size, context.config.format);

			let data = vec![[10, 10, 10, 255]; 10 * 10]
				.into_iter()
				.flatten()
				.collect::<Vec<_>>();

			texture.write_data(context.queue, &data);

			let bind_group =
				texture.bind_group(context.device, context.bind_group_layout, context.sampler);

			let mut vertices = outer.corners().to_vec();
			vertices.extend(inner.corners());

			let mut indices = vec![];

			indices.extend([0, 1, 4]);
			indices.extend([1, 5, 4]);

			indices.extend([0, 4, 2]);
			indices.extend([4, 6, 2]);

			indices.extend([2, 6, 7]);
			indices.extend([2, 7, 3]);

			indices.extend([1, 7, 5]);
			indices.extend([1, 3, 7]);

			let border = RenderedMesh::new(&context.device, &vertices, &indices, bind_group);

			(
				border,
				self.value
					.get_renderable(context, inner),
			)
		}

		fn width_hint(&self) -> SizeHint {
			SizeHint::Sum(vec![
				self.value.width_hint(),
				SizeHint::Physical(self.width * 2),
			])
		}

		fn height_hint(&self) -> SizeHint {
			SizeHint::Sum(vec![
				self.value.height_hint(),
				SizeHint::Physical(self.width * 2),
			])
		}
	}

	macro_rules! tuple_impl {
    ($($name:ident),*) => {
        impl<$($name: Widget),*> Widget for ($($name),*) {
        	type Renderable = ($($name::Renderable),*);

        	fn get_renderable(&mut self, context: &mut crate::context::Context<WidgetContext>, view: crate::view::View) -> Self::Renderable {
        		let view = view.from_size_hints(self.width_hint(), self.height_hint());
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				($(<$name as Widget>::get_renderable([<$name:snake>], context, view.clone())),*)
    			}
    		}

    		fn width_hint(&self) -> crate::view::SizeHint {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				crate::view::SizeHint::Max(vec![$(<$name as Widget>::width_hint([<$name:snake>])),*])
    			}
    		}

    		fn height_hint(&self) -> crate::view::SizeHint {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				crate::view::SizeHint::Max(vec![$(<$name as Widget>::height_hint([<$name:snake>])),*])
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
