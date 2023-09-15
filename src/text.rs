use ab_glyph::{Font as Font2, FontRef};
use wgpu::TextureUsages;

pub struct Font {
	font: FontRef<'static>,
	layout: wgpu::BindGroupLayout,
}

impl Font {
	pub fn new(font: FontRef<'static>, device: &wgpu::Device) -> Self {
		let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
			label: None,
			entries: &[
				wgpu::BindGroupLayoutEntry {
					binding: 0,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Texture {
						sample_type: wgpu::TextureSampleType::Float { filterable: true },
						view_dimension: wgpu::TextureViewDimension::D2,
						multisampled: false,
					},
					count: None,
				},
				wgpu::BindGroupLayoutEntry {
					binding: 1,
					visibility: wgpu::ShaderStages::FRAGMENT,
					ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
					count: None,
				},
			],
		});

		Self { font, layout }
	}

	pub fn binding_layout(&self) -> &wgpu::BindGroupLayout {
		&self.layout
	}

	pub fn rasterize(
		&self,
		value: char,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
	) -> wgpu::BindGroup {
		let glyph = self
			.font
			.glyph_id(value)
			.with_scale(100.0);

		let size = self.font.glyph_bounds(&glyph);
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

		self.font
			.outline_glyph(glyph)
			.unwrap()
			.draw(|x, y, c| {
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

		let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
			label: None,
			address_mode_u: wgpu::AddressMode::ClampToEdge,
			address_mode_v: wgpu::AddressMode::ClampToEdge,
			address_mode_w: wgpu::AddressMode::ClampToEdge,
			mag_filter: wgpu::FilterMode::Linear,
			min_filter: wgpu::FilterMode::Linear,
			mipmap_filter: wgpu::FilterMode::Nearest,
			..Default::default()
		});

		device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout: &self.layout,
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
		})
	}
}
