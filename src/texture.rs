pub struct Texture {
	size: wgpu::Extent3d,
	texture: wgpu::Texture,
	view: wgpu::TextureView,
}

impl Texture {
	pub fn new(device: &wgpu::Device, size: wgpu::Extent3d, format: wgpu::TextureFormat) -> Self {
		let texture = device.create_texture(&wgpu::TextureDescriptor {
			label: None,
			size,
			mip_level_count: 1,
			sample_count: 1,
			dimension: wgpu::TextureDimension::D2,
			format,
			usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
			view_formats: &[],
		});

		let view = texture.create_view(&Default::default());

		Self {
			size,
			texture,
			view,
		}
	}

	pub fn write_data(&mut self, queue: &wgpu::Queue, data: &[u8]) {
		queue.write_texture(
			wgpu::ImageCopyTexture {
				texture: &self.texture,
				mip_level: 0,
				origin: wgpu::Origin3d { x: 0, y: 0, z: 0 },
				aspect: wgpu::TextureAspect::All,
			},
			data,
			wgpu::ImageDataLayout {
				offset: 0,
				bytes_per_row: Some(4 * self.size.width),
				rows_per_image: Some(self.size.height),
			},
			self.size,
		)
	}

	pub fn bind_group(
		&self,
		device: &wgpu::Device,
		layout: &wgpu::BindGroupLayout,
		sampler: &wgpu::Sampler,
	) -> wgpu::BindGroup {
		device.create_bind_group(&wgpu::BindGroupDescriptor {
			label: None,
			layout,
			entries: &[
				wgpu::BindGroupEntry {
					binding: 0,
					resource: wgpu::BindingResource::TextureView(&self.view),
				},
				wgpu::BindGroupEntry {
					binding: 1,
					resource: wgpu::BindingResource::Sampler(sampler),
				},
			],
		})
	}
}
