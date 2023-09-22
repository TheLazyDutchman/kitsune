use winit::dpi::{PhysicalPosition, PhysicalSize};

use crate::render::Vertex;

#[derive(Debug, Clone, Copy)]
pub struct GlobalView {
	size: PhysicalSize<u32>,
}

impl GlobalView {
	pub fn new(size: PhysicalSize<u32>) -> Self {
		Self { size }
	}

	pub fn view(&self, size: PhysicalSize<u32>, offset: PhysicalPosition<u32>) -> View {
		View {
			global: *self,
			size,
			offset,
		}
	}
}

#[derive(Debug, Clone)]
pub struct View {
	global: GlobalView,
	size: PhysicalSize<u32>,
	offset: PhysicalPosition<u32>,
}

impl View {
	fn virtualize_x(&self, x: u32) -> f32 {
		x as f32 / self.size.width as f32
	}

	fn physical_x(&self, x: f32) -> u32 {
		(x * self.size.width as f32) as u32
	}

	fn virtualize_y(&self, y: u32) -> f32 {
		y as f32 / self.size.height as f32
	}

	fn physical_y(&self, y: f32) -> u32 {
		(y * self.size.height as f32) as u32
	}

	pub fn virtualize(&self, pos: PhysicalPosition<u32>) -> VirtualPosition {
		VirtualPosition::new(self.virtualize_x(pos.x), self.virtualize_y(pos.y))
	}

	pub fn physical_width_hint(&self, hint: SizeHint) -> Option<u32> {
		match hint {
			SizeHint::None => None,
			SizeHint::Physical(value) => Some(value),
			SizeHint::Virtual(value) => Some(self.physical_x(value)),
			SizeHint::Max(value) => value
				.into_iter()
				.flat_map(|x| self.physical_width_hint(x))
				.reduce(|a, b| a.max(b)),
			SizeHint::Min(value) => value
				.into_iter()
				.flat_map(|x| self.physical_width_hint(x))
				.reduce(|a, b| a.min(b)),
			SizeHint::Sum(value) => value
				.into_iter()
				.flat_map(|x| self.physical_width_hint(x))
				.reduce(|a, b| a + b),
		}
	}

	pub fn physical_height_hint(&self, hint: SizeHint) -> Option<u32> {
		match hint {
			SizeHint::None => None,
			SizeHint::Physical(value) => Some(value),
			SizeHint::Virtual(value) => Some(self.physical_y(value)),
			SizeHint::Max(value) => value
				.into_iter()
				.flat_map(|x| self.physical_height_hint(x))
				.reduce(|a, b| a.max(b)),
			SizeHint::Min(value) => value
				.into_iter()
				.flat_map(|x| self.physical_height_hint(x))
				.reduce(|a, b| a.min(b)),
			SizeHint::Sum(value) => value
				.into_iter()
				.flat_map(|x| self.physical_height_hint(x))
				.reduce(|a, b| a + b),
		}
	}

	pub fn virtualize_width_hint(&self, hint: SizeHint) -> Option<f32> {
		match hint {
			SizeHint::None => None,
			SizeHint::Physical(value) => Some(self.virtualize_x(value)),
			SizeHint::Virtual(value) => Some(value),
			SizeHint::Max(value) => value
				.into_iter()
				.flat_map(|x| self.virtualize_width_hint(x))
				.reduce(|a, b| a.max(b)),
			SizeHint::Min(value) => value
				.into_iter()
				.flat_map(|x| self.virtualize_width_hint(x))
				.reduce(|a, b| a.min(b)),
			SizeHint::Sum(value) => value
				.into_iter()
				.flat_map(|x| self.virtualize_width_hint(x))
				.reduce(|a, b| a + b),
		}
	}

	pub fn virtualize_height_hint(&self, hint: SizeHint) -> Option<f32> {
		match hint {
			SizeHint::None => None,
			SizeHint::Physical(value) => Some(self.virtualize_y(value)),
			SizeHint::Virtual(value) => Some(value),
			SizeHint::Max(value) => value
				.into_iter()
				.flat_map(|x| self.virtualize_height_hint(x))
				.reduce(|a, b| a.max(b)),
			SizeHint::Min(value) => value
				.into_iter()
				.flat_map(|x| self.virtualize_height_hint(x))
				.reduce(|a, b| a.min(b)),
			SizeHint::Sum(value) => value
				.into_iter()
				.flat_map(|x| self.virtualize_height_hint(x))
				.reduce(|a, b| a + b),
		}
	}

	pub fn globalize(&self, pos: VirtualPosition) -> GlobalPosition {
		let x = ((pos.x * self.size.width as f32) + self.offset.x as f32)
			/ self.global.size.width as f32;
		let y = ((pos.y * self.size.height as f32) + self.offset.y as f32)
			/ self.global.size.height as f32;

		// Wgpu uses a coordinate system where bottom-left is (-1.0, -1.0) and the top-right is
		// (1.0, 1.0).
		// Since we use a coordinate system where top-left is (0.0, 0.0) and bottom-right is (1.0,
		// 1.0) we need to convert these coordinates to wgpu`s coordinates
		let x = x * 2.0 - 1.0;
		let y = y * -2.0 + 1.0;

		GlobalPosition { x, y }
	}

	pub fn split_row(self, hints: Vec<SizeHint>) -> Vec<Self> {
		let mut values = vec![];
		let mut offset = 0;

		for hint in hints {
			// TODO: I do not yet know how to handle an unknown size hint
			let width = self
				.physical_width_hint(hint)
				.unwrap_or(0);

			let size = PhysicalSize::new(width, self.size.height);
			values.push(self.global.view(
				size,
				PhysicalPosition::new(self.offset.x + offset, self.offset.y),
			));

			offset += width;
		}
		values
	}

	pub fn split_column(self, hints: Vec<SizeHint>) -> Vec<Self> {
		let mut values = vec![];
		let mut offset = 0;

		for hint in hints {
			// TODO: I do not yet know how to handle an unknown size hint
			let height = self
				.physical_width_hint(hint)
				.unwrap_or(0);

			let size = PhysicalSize::new(self.size.width, height);
			values.push(self.global.view(
				size,
				PhysicalPosition::new(self.offset.x, self.offset.y + offset),
			));

			offset += height;
		}
		values
	}

	pub fn bordered(self, width: u32) -> (Self, Self) {
		let size = PhysicalSize::new(self.size.width - 2 * width, self.size.height - 2 * width);
		let offset = PhysicalPosition::new(self.offset.x + width, self.offset.y + width);
		let inner = self.global.view(size, offset);
		(self, inner)
	}

	pub fn from_size_hints(self, width: SizeHint, height: SizeHint) -> View {
		let size = PhysicalSize::new(
			self.physical_width_hint(width)
				.unwrap_or(self.size.width)
				.min(self.width()),
			self.physical_height_hint(height)
				.unwrap_or(self.size.width)
				.min(self.height()),
		);
		let offset = PhysicalPosition::new(self.offset.x, self.offset.y);
		self.global.view(size, offset)
	}

	/// Get the vertices of the four corners of this view.
	///
	/// they are ordered counter clock wise.
	///
	pub fn corners(&self) -> [Vertex; 4] {
		[
			Vertex::new(self.globalize(VirtualPosition::new(0.0, 0.0)), [0.0, 0.0]),
			Vertex::new(self.globalize(VirtualPosition::new(0.0, 1.0)), [0.0, 1.0]),
			Vertex::new(self.globalize(VirtualPosition::new(1.0, 1.0)), [1.0, 1.0]),
			Vertex::new(self.globalize(VirtualPosition::new(1.0, 0.0)), [1.0, 0.0]),
		]
	}

	pub fn width(&self) -> u32 {
		self.size.width
	}

	pub fn height(&self) -> u32 {
		self.size.height
	}
}

#[derive(Debug, Clone, Copy)]
pub struct VirtualPosition {
	x: f32,
	y: f32,
}

impl VirtualPosition {
	pub fn new(x: f32, y: f32) -> Self {
		Self { x, y }
	}
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GlobalPosition {
	x: f32,
	y: f32,
}

pub enum SizeHint {
	None,
	Physical(u32),
	Virtual(f32),
	Max(Vec<SizeHint>),
	Min(Vec<SizeHint>),
	Sum(Vec<SizeHint>),
}
