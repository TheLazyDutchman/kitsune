use winit::dpi::PhysicalSize;

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

	fn width_hint(&self, _context: &Context<WidgetContext>) -> SizeHint {
		SizeHint::None
	}

	fn height_hint(&self, _context: &Context<WidgetContext>) -> SizeHint {
		SizeHint::None
	}

	fn cached(self) -> Cached<Self>
	where
		Self: Sized,
	{
		Cached::new(self)
	}

	fn bordered(self, size: u32) -> Bordered<Self>
	where
		Self: Sized,
	{
		Bordered::new(self, size)
	}

	fn resize(&mut self, _new_size: PhysicalSize<u32>) {}
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
		struct $name:ident<T $(:$bound:ident)?> {
			$value:ident: $ty:ty
			$(, $field:ident: $field_ty:ty)*
			$(,#default $($default_field:ident: $default_ty:ty),*)?
		}

		$(#on_mut $mut_fn:item)?
	) => {
		pub struct $name<T $(:$bound)?> {
			$value: $ty,
			$($field:$field_ty,)*
			$($($default_field:$default_ty),*)?
		}

		impl<T> $name<T> where $(T:$bound)? {
			pub fn new($value: $ty, $($field:$field_ty),*) -> Self {
				Self {
					$value,
					$($field),*
					$($($default_field: <$default_ty>::default()),*)?
				}
			}
		}

		impl<T> std::ops::Deref for $name<T> where $(T:$bound)? {
			type Target = $ty;

			fn deref(&self) -> &Self::Target {
				&self.$value
			}
		}

		impl<T> std::ops::DerefMut for $name<T> where $(T:$bound)? {
			wrapper!(on_mut $value $($mut_fn)?);
		}
	};
	(on_mut $value:ident $mut_fn:item) => {
		$mut_fn
	};
	(on_mut $value:ident) => {
		fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.$value
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
		size: u32
	}
}

wrapper! {
	struct Cached<T: Widget> {
		value: T,

		#default
		cached: Option<std::rc::Rc<T::Renderable>>
	}

	#on_mut
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.cached = None;
		&mut self.value
	}
}

mod impls {
	use paste::paste;

	use super::*;
	use crate::{context::Context, texture::Texture};

	#[cfg(feature = "text")]
	impl Widget for char {
		type Renderable = Option<RenderedMesh>;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			let bind_group = context.font.rasterize(
				context.font.glyph(*self),
				context.device,
				context.config.format,
				context.queue,
				context.sampler,
				context.bind_group_layout,
			)?;

			let view = view.from_size_hints(self.width_hint(context), self.height_hint(context));
			let vertices = view.corners();

			let indices = [0, 1, 2, 2, 3, 0];

			Some(RenderedMesh::new(
				context.device,
				&vertices,
				&indices,
				bind_group,
			))
		}

		fn width_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Physical(
				context
					.font
					.glyph(*self)
					.size()
					.width() as u32,
			)
		}

		fn height_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Physical(
				context
					.font
					.glyph(*self)
					.size()
					.height() as u32,
			)
		}
	}

	#[cfg(feature = "text")]
	impl Widget for String {
		type Renderable = Vec<Option<RenderedMesh>>;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			Row::new(self.chars().collect()).get_renderable(context, view)
		}

		fn width_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Sum(
				self.chars()
					.into_iter()
					.map(|x| x.width_hint(context))
					.collect(),
			)
		}

		fn height_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Max(
				self.chars()
					.into_iter()
					.map(|x| x.height_hint(context))
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
			let view = view.from_size_hints(self.width_hint(context), self.height_hint(context));
			let views = view.split_row(
				self.values
					.iter()
					.map(|x| x.width_hint(context))
					.collect(),
			);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}

		fn width_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Sum(
				self.values
					.iter()
					.map(|x| x.width_hint(context))
					.collect(),
			)
		}

		fn height_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Max(
				self.values
					.iter()
					.map(|x| x.height_hint(context))
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
			let view = view.from_size_hints(self.width_hint(context), self.height_hint(context));
			let views = view.split_column(
				self.values
					.iter()
					.map(|x| x.height_hint(context))
					.collect(),
			);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}

		fn width_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Max(
				self.values
					.iter()
					.map(|x| x.width_hint(context))
					.collect(),
			)
		}

		fn height_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Sum(
				self.values
					.iter()
					.map(|x| x.height_hint(context))
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
			let view = view.from_size_hints(self.width_hint(context), self.height_hint(context));
			let (outer, inner) = view.bordered(self.size);

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

			indices.extend([0, 4, 3]);
			indices.extend([4, 7, 3]);

			indices.extend([3, 7, 6]);
			indices.extend([3, 6, 2]);

			indices.extend([1, 6, 5]);
			indices.extend([1, 2, 6]);

			let border = RenderedMesh::new(&context.device, &vertices, &indices, bind_group);

			(
				border,
				self.value
					.get_renderable(context, inner),
			)
		}

		fn width_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Sum(vec![
				self.value.width_hint(context),
				SizeHint::Physical(self.size * 2),
			])
		}

		fn height_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			SizeHint::Sum(vec![
				self.value
					.height_hint(context),
				SizeHint::Physical(self.size * 2),
			])
		}

		fn resize(&mut self, new_size: PhysicalSize<u32>) {
			self.value
				.resize(PhysicalSize::new(
					new_size.width - self.size * 2,
					new_size.height - self.size * 2,
				));
		}
	}

	impl<T> Widget for Cached<T>
	where
		T: Widget,
	{
		type Renderable = std::rc::Rc<T::Renderable>;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			if let Some(ref renderable) = self.cached {
				renderable.clone()
			} else {
				let renderable = std::rc::Rc::new(
					self.value
						.get_renderable(context, view),
				);
				self.cached = Some(renderable.clone());
				renderable
			}
		}

		fn width_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			(**self).width_hint(context)
		}

		fn height_hint(&self, context: &Context<WidgetContext>) -> SizeHint {
			(**self).height_hint(context)
		}

		fn resize(&mut self, new_size: PhysicalSize<u32>) {
			(**self).resize(new_size);
		}
	}

	macro_rules! tuple_impl {
    ($($name:ident),*) => {
        impl<$($name: Widget),*> Widget for ($($name),*) {
        	type Renderable = ($($name::Renderable),*);

        	fn get_renderable(&mut self, context: &mut crate::context::Context<WidgetContext>, view: crate::view::View) -> Self::Renderable {
        		let view = view.from_size_hints(self.width_hint(context), self.height_hint(context));
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				($(<$name as Widget>::get_renderable([<$name:snake>], context, view.clone())),*)
    			}
    		}

    		fn width_hint(&self, context: &crate::context::Context<crate::widget::WidgetContext>) -> crate::view::SizeHint {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				crate::view::SizeHint::Max(vec![$(<$name as Widget>::width_hint([<$name:snake>], context)),*])
    			}
    		}

    		fn height_hint(&self, context: &crate::context::Context<crate::widget::WidgetContext>) -> crate::view::SizeHint {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				crate::view::SizeHint::Max(vec![$(<$name as Widget>::height_hint([<$name:snake>], context)),*])
    			}
    		}

			fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
				paste! {
    				let ($([<$name:snake>]),*) = self;
    				$(<$name as Widget>::resize([<$name:snake>], new_size);)*
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
