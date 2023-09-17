use ab_glyph::{Font as Font2, FontRef};
use wgpu::TextureUsages;

pub struct Font {
	font: FontRef<'static>,
}

impl Font {
	pub fn new(font: FontRef<'static>) -> Self {
		Self { font }
	}

	pub fn rasterize(
		&self,
		value: char,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
		sampler: &wgpu::Sampler,
		layout: &wgpu::BindGroupLayout,
	) -> Option<wgpu::BindGroup> {
		let glyph = self
			.font
			.glyph_id(value)
			.with_scale(100.0);

		let size = self.font.glyph_bounds(&glyph);

		let outlined_glyph = self
			.font
			.outline_glyph(glyph)?;

		let size = wgpu::Extent3d {
			width: size.width() as u32,
			height: size.height() as u32,
			depth_or_array_layers: 1,
		};

		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: None,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format: wgpu::TextureFormat::Bgra8UnormSrgb,
			usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		});

		let mut data = vec![0; (4 * size.width * size.height) as usize];

		outlined_glyph.draw(|x, y, c| {
			let color_value = 0;
			let alpha_value = (255.0 * c) as u8;

			let index = size.width * y + x;
			let index = index as usize * 4;

			data[index] = color_value;
			data[index + 1] = color_value;
			data[index + 2] = color_value;
			data[index + 3] = alpha_value;
		});

		queue.write_texture(
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

		let view = texture.create_view(&Default::default());

		Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(&sampler),
				},
			],
		}))
	}
}
