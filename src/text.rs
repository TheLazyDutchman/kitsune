use ab_glyph::{Font as Font2, FontRef};

use crate::texture::Texture;

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
		format: wgpu::TextureFormat,
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

		let mut texture = Texture::new(device, size, format);

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

		texture.write_data(queue, &data);

		Some(texture.bind_group(device, layout, sampler))
	}
}
