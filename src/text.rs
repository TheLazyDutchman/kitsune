use ab_glyph::{Font as Font2, FontRef, Rect};

use crate::texture::Texture;

pub struct Font {
	font: FontRef<'static>,
}

impl Font {
	pub fn new(font: FontRef<'static>) -> Self {
		Self { font }
	}

	pub fn glyph(&self, value: char) -> Glyph {
		let glyph = self
			.font
			.glyph_id(value)
			.with_scale(100.0);
		let size = self.font.glyph_bounds(&glyph);
		Glyph { glyph, size }
	}

	pub fn rasterize(
		&self,
		glyph: Glyph,
		device: &wgpu::Device,
		format: wgpu::TextureFormat,
		queue: &wgpu::Queue,
		sampler: &wgpu::Sampler,
		layout: &wgpu::BindGroupLayout,
	) -> Option<wgpu::BindGroup> {
		let size = glyph.size();

		let outlined_glyph = self
			.font
			.outline_glyph(glyph.glyph)?;

		let size = wgpu::Extent3d {
			width: size.width() as u32,
			height: size.height() as u32,
			depth_or_array_layers: 1,
		};

		let mut texture = Texture::new(device, size, format);

		let mut data = vec![0; (4 * size.width * size.height) as usize];

		let mut max_y = 0;

		// TODO: I do not know how to correctly do the offset of characters otherwise.
		outlined_glyph.draw(|_, y, _| {
			if y > max_y {
				max_y = y
			}
		});

		let y_offset = (size.height - 1) - max_y;

		outlined_glyph.draw(|x, mut y, c| {
			let color_value = 0;
			let alpha_value = (255.0 * c) as u8;

			y += y_offset;

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

#[derive(Debug, Clone)]
pub struct Glyph {
	glyph: ab_glyph::Glyph,
	size: Rect,
}

impl Glyph {
	pub fn size(&self) -> Rect {
		self.size
	}
}
