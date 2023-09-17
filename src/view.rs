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
	pub fn virtualize(&self, pos: PhysicalPosition<u32>) -> VirtualPosition {
		let x = pos.x as f32 / self.size.width as f32;
		let y = pos.y as f32 / self.size.height as f32;

		VirtualPosition { x, y }
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

	pub fn split_row(self, count: u32) -> Vec<Self> {
		let width = self.size.width / count;
		let size = PhysicalSize::new(width, self.size.height);

		let mut values = vec![];
		for i in 0..count {
			values.push(self.global.view(
				size,
				PhysicalPosition::new(self.offset.x + i * width, self.offset.y),
			));
		}
		values
	}

	pub fn split_column(self, count: u32) -> Vec<Self> {
		let height = self.size.height / count;
		let size = PhysicalSize::new(self.size.width, height);

		let mut values = vec![];
		for i in 0..count {
			values.push(self.global.view(
				size,
				PhysicalPosition::new(self.offset.x, self.offset.y + i * height),
			));
		}
		values
	}

	pub fn bordered(self, width: u32) -> (Self, Self) {
		let size = PhysicalSize::new(self.size.width - 2 * width, self.size.height - 2 * width);
		let offset = PhysicalPosition::new(self.offset.x + width, self.offset.y + width);
		let inner = self.global.view(size, offset);
		(self, inner)
	}

	pub fn corners(&self) -> [Vertex; 4] {
		[
			Vertex::new(self.globalize(VirtualPosition::new(0.0, 0.0)), [0.0, 0.0]),
			Vertex::new(self.globalize(VirtualPosition::new(0.0, 1.0)), [0.0, 1.0]),
			Vertex::new(self.globalize(VirtualPosition::new(1.0, 0.0)), [1.0, 0.0]),
			Vertex::new(self.globalize(VirtualPosition::new(1.0, 1.0)), [1.0, 1.0]),
		]
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
