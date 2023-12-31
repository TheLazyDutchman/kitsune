use itertools::Itertools;
use winit::{dpi::PhysicalSize, event::WindowEvent};

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

	fn width_hint(&self, _context: &Context<WidgetContext>, _view: &View) -> SizeHint {
		SizeHint::None
	}

	fn height_hint(&self, _context: &Context<WidgetContext>, _view: &View) -> SizeHint {
		SizeHint::None
	}

	fn resize(&mut self, _new_size: PhysicalSize<u32>) {}
	fn handle(&mut self, _event: &WindowEvent) {}

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
	struct WrappingRow<T> {
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

			let width = self.width_hint(context, &view);
			let height = self.height_hint(context, &view);
			let view = view.from_size_hints(width, height);
			let vertices = view.corners();

			let indices = [0, 1, 2, 2, 3, 0];

			Some(RenderedMesh::new(
				context.device,
				&vertices,
				&indices,
				bind_group,
			))
		}

		fn width_hint(&self, context: &Context<WidgetContext>, _view: &View) -> SizeHint {
			SizeHint::Physical(
				context
					.font
					.glyph(*self)
					.size()
					.width() as u32,
			)
		}

		fn height_hint(&self, context: &Context<WidgetContext>, _view: &View) -> SizeHint {
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
		type Renderable = <WrappingRow<char> as Widget>::Renderable;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			WrappingRow::new(self.chars().collect()).get_renderable(context, view)
		}

		fn width_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			WrappingRow::new(self.chars().collect()).width_hint(context, view)
		}

		fn height_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			WrappingRow::new(self.chars().collect()).height_hint(context, view)
		}
	}

	impl<'a, T> Widget for &'a mut T
	where
		T: Widget,
	{
		type Renderable = T::Renderable;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			(**self).get_renderable(context, view)
		}

		fn resize(&mut self, new_size: PhysicalSize<u32>) {
			(**self).resize(new_size);
		}

		fn handle(&mut self, event: &WindowEvent) {
			(**self).handle(event);
		}

		fn width_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			(**self).width_hint(context, view)
		}

		fn height_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			(**self).height_hint(context, view)
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
			let width = self.width_hint(context, &view);
			let height = self.height_hint(context, &view);
			let view = view.from_size_hints(width, height);

			let hints = self
				.values
				.iter()
				.map(|x| x.width_hint(context, &view))
				.collect();
			let views = view.split_row(hints);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}

		fn width_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			SizeHint::Sum(
				self.values
					.iter()
					.map(|x| x.width_hint(context, &view))
					.collect(),
			)
		}

		fn height_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			SizeHint::Max(
				self.values
					.iter()
					.map(|x| x.height_hint(context, &view))
					.collect(),
			)
		}

		fn handle(&mut self, event: &WindowEvent) {
			for value in &mut self.values {
				value.handle(event);
			}
		}
	}

	impl<T> Widget for WrappingRow<T>
	where
		T: Widget,
	{
		type Renderable = Vec<Vec<<T as Widget>::Renderable>>;

		fn get_renderable(
			&mut self,
			context: &mut Context<WidgetContext>,
			view: View,
		) -> Self::Renderable {
			let mut columns = vec![];
			let mut current_row = vec![];

			let mut offset = 0;
			for value in &mut self.values {
				offset += view
					.physical_width_hint(value.width_hint(context, &view))
					.unwrap_or(0);

				if offset > view.width() {
					columns.push(Row::new(std::mem::take(&mut current_row)));
					offset = 0;
				}

				current_row.push(value);
			}

			if !current_row.is_empty() {
				columns.push(Row::new(current_row));
			}

			Column::new(columns).get_renderable(context, view)
		}

		fn handle(&mut self, event: &WindowEvent) {
			for value in &mut self.values {
				value.handle(event);
			}
		}

		fn width_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			let sum = self
				.values
				.iter()
				.map(|x| x.width_hint(context, &view))
				.collect();
			SizeHint::Min(vec![SizeHint::Sum(sum), SizeHint::Physical(view.width())])
		}

		fn height_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			let heights = self
				.values
				.iter()
				.peekable()
				.batching(|x| {
					let mut width = 0;
					let mut heights = vec![];
					while let Some(val) = x.peek() {
						width += view
							.physical_width_hint(val.width_hint(context, view))
							.unwrap_or(0);
						if width > view.width() {
							break;
						}

						heights.push(val.height_hint(context, view));
						x.next();
					}

					if heights.is_empty() {
						None
					} else {
						Some(heights)
					}
				});
			SizeHint::Sum(
				heights
					.map(|x| SizeHint::Max(x))
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
			let width = self.width_hint(context, &view);
			let height = self.height_hint(context, &view);
			let view = view.from_size_hints(width, height);

			let hints = self
				.values
				.iter()
				.map(|x| x.height_hint(context, &view))
				.collect();
			let views = view.split_column(hints);

			self.values
				.iter_mut()
				.zip(views)
				.map(|(w, v)| w.get_renderable(context, v))
				.collect()
		}

		fn width_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			SizeHint::Max(
				self.values
					.iter()
					.map(|x| x.width_hint(context, view))
					.collect(),
			)
		}

		fn height_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			SizeHint::Sum(
				self.values
					.iter()
					.map(|x| x.height_hint(context, view))
					.collect(),
			)
		}

		fn handle(&mut self, event: &WindowEvent) {
			for value in &mut self.values {
				value.handle(event);
			}
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
			let width = self.width_hint(context, &view);
			let height = self.height_hint(context, &view);
			let view = view.from_size_hints(width, height);
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

		fn width_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			SizeHint::Sum(vec![
				self.value
					.width_hint(context, view),
				SizeHint::Physical(self.size * 2),
			])
		}

		fn height_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			SizeHint::Sum(vec![
				self.value
					.height_hint(context, view),
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

		fn width_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			(**self).width_hint(context, view)
		}

		fn height_hint(&self, context: &Context<WidgetContext>, view: &View) -> SizeHint {
			(**self).height_hint(context, view)
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
        		let width = self.width_hint(context, &view);
        		let height = self.height_hint(context, &view);
        		let view = view.from_size_hints(width, height);
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				($(<$name as Widget>::get_renderable([<$name:snake>], context, view.clone())),*)
    			}
    		}

    		fn width_hint(&self, context: &crate::context::Context<crate::widget::WidgetContext>, view: &crate::view::View) -> crate::view::SizeHint {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				crate::view::SizeHint::Max(vec![$(<$name as Widget>::width_hint([<$name:snake>], context, view)),*])
    			}
    		}

    		fn height_hint(&self, context: &crate::context::Context<crate::widget::WidgetContext>, view: &crate::view::View) -> crate::view::SizeHint {
    			paste! {
    				let ($([<$name:snake>]),*) = self;
    				crate::view::SizeHint::Max(vec![$(<$name as Widget>::height_hint([<$name:snake>], context, view)),*])
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
