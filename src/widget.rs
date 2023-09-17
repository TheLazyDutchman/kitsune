use crate::{
	context::Context,
	render::{Render, RenderedMesh},
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
				context.queue,
				context.sampler,
				context.bind_group_layout,
			) else {
				return None;
			};

			let vertices = view.corners();

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
			let (outer, inner) = view.bordered(self.width);

			let size = wgpu::Extent3d {
				width: 10,
				height: 10,
				depth_or_array_layers: 1,
			};

			let texture = context
				.device
				.create_texture(&wgpu::TextureDescriptor {
					label: None,
					size,
					mip_level_count: 1,
					sample_count: 1,
					dimension: wgpu::TextureDimension::D2,
					format: context.config.format,
					usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
					view_formats: &[],
				});

			let view = texture.create_view(&Default::default());

			let data = vec![[10, 10, 10, 255]; 10 * 10]
				.into_iter()
				.flatten()
				.collect::<Vec<_>>();

			context.queue.write_texture(
				wgpu::ImageCopyTexture {
					texture: &texture,
					mip_level: 0,
					origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
					aspect: wgpu::TextureAspect::All,
				},
				&data,
				wgpu::ImageDataLayout {
					offset: 0,
					bytes_per_row: Some(4 * size.width),
					rows_per_image: Some(size.height),
				},
				size,
			);

			let bind_group = context
				.device
				.create_bind_group(&wgpu::BindGroupDescriptor {
					label: None,
					layout: context.bind_group_layout,
					entries: &[
						wgpu::BindGroupEntry {
							binding: 0,
							resource: wgpu::BindingResource::TextureView(&view),
						},
						wgpu::BindGroupEntry {
							binding: 1,
							resource: wgpu::BindingResource::Sampler(context.sampler),
						},
					],
				});

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
