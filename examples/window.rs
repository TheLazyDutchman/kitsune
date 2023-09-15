use ab_glyph::FontRef;
use wgpu::{
	include_wgsl,
	util::{BufferInitDescriptor, DeviceExt},
	vertex_attr_array, BindGroup, BufferUsages, Color, ColorTargetState, ColorWrites,
	CompositeAlphaMode, Device, FragmentState, IndexFormat, LoadOp, MultisampleState, Operations,
	PipelineLayoutDescriptor, PresentMode, PrimitiveState, PrimitiveTopology, Queue,
	RenderPassDescriptor, RenderPipeline, RenderPipelineDescriptor, Surface, SurfaceConfiguration,
	TextureFormat, TextureUsages, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode,
};
use winit::{
	event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
	event_loop::EventLoop,
	window::Window,
};

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 2],
	uv: [f32; 2],
}

impl Vertex {
	const LAYOUT: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x2, 1 => Float32x2];

	fn layout() -> VertexBufferLayout<'static> {
		VertexBufferLayout {
			array_stride: std::mem::size_of::<Self>() as u64,
			step_mode: VertexStepMode::Vertex,
			attributes: &Self::LAYOUT,
		}
	}

	fn new(position: [f32; 2], uv: [f32; 2]) -> Self {
		Self { position, uv }
	}
}

#[tokio::main]
async fn main() {
	let event_loop = EventLoop::new();

	let window = Window::new(&event_loop).unwrap();

	let size = window.inner_size();

	let instance = wgpu::Instance::new(Default::default());

	let surface = unsafe { instance.create_surface(&window) }.unwrap();

	let adaper = instance
		.request_adapter(&Default::default())
		.await
		.unwrap();

	let (device, queue) = adaper
		.request_device(&Default::default(), None)
		.await
		.unwrap();

	surface.configure(
		&device,
		&SurfaceConfiguration {
			usage: TextureUsages::RENDER_ATTACHMENT,
			format: TextureFormat::Bgra8UnormSrgb,
			width: size.width,
			height: size.height,
			present_mode: PresentMode::AutoVsync,
			alpha_mode: CompositeAlphaMode::Auto,
			view_formats: vec![],
		},
	);

	let font = FontRef::try_from_slice(include_bytes!("../res/Roboto/Roboto-Medium.ttf")).unwrap();
	let font = kitsune::text::Font::new(font, &device);

	let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
		label: Some("Pipeline Layout"),
		bind_group_layouts: &[&font.binding_layout()],
		push_constant_ranges: &[],
	});

	let shader = device.create_shader_module(include_wgsl!("shader.wgsl"));

	let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
		label: Some("Render Pipeline"),
		layout: Some(&layout),
		vertex: VertexState {
			module: &shader,
			entry_point: "vs_main",
			buffers: &[Vertex::layout()],
		},
		primitive: PrimitiveState {
			topology: PrimitiveTopology::TriangleList,
			strip_index_format: None,
			front_face: wgpu::FrontFace::Ccw,
			cull_mode: Some(wgpu::Face::Back),
			unclipped_depth: false,
			polygon_mode: wgpu::PolygonMode::Fill,
			conservative: false,
		},
		depth_stencil: None,
		multisample: MultisampleState {
			count: 1,
			mask: !0,
			alpha_to_coverage_enabled: false,
		},
		fragment: Some(FragmentState {
			module: &shader,
			entry_point: "fs_main",
			targets: &[Some(ColorTargetState {
				format: TextureFormat::Bgra8UnormSrgb,
				blend: None,
				write_mask: ColorWrites::ALL,
			})],
		}),
		multiview: None,
	});

	let bind_group = font.rasterize('9', &device, &queue);

	event_loop.run(move |event, _, controlflow| match event {
		Event::WindowEvent {
			window_id,
			event:
				WindowEvent::CloseRequested
				| WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							state: ElementState::Pressed,
							virtual_keycode: Some(VirtualKeyCode::Escape),
							..
						},
					..
				},
		} if window.id() == window_id => {
			controlflow.set_exit();
		}
		Event::MainEventsCleared => {
			window.request_redraw();
		}
		Event::RedrawRequested(window_id) if window.id() == window_id => {
			draw(&surface, &device, &pipeline, &bind_group, &queue);
		}
		_ => {}
	});
}

fn draw(
	surface: &Surface,
	device: &Device,
	pipeline: &RenderPipeline,
	bind_group: &BindGroup,
	queue: &Queue,
) {
	let output = surface
		.get_current_texture()
		.unwrap();
	let view = output
		.texture
		.create_view(&Default::default());

	let vertices = [
		Vertex::new([-1.0, 1.0], [0.0, 0.0]),
		Vertex::new([-1.0, -1.0], [0.0, 1.0]),
		Vertex::new([1.0, 1.0], [1.0, 0.0]),
		Vertex::new([1.0, -1.0], [1.0, 1.0]),
	];
	let indices: [u16; 6] = [0, 1, 2, 1, 3, 2];

	let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
		label: Some("Vertex Buffer"),
		contents: bytemuck::cast_slice(&vertices),
		usage: BufferUsages::VERTEX,
	});

	let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
		label: Some("Index Buffer"),
		contents: bytemuck::cast_slice(&indices),
		usage: BufferUsages::INDEX,
	});

	let mut encoder = device.create_command_encoder(&Default::default());

	{
		let mut pass = encoder.begin_render_pass(&RenderPassDescriptor {
			label: Some("Render Pass"),
			color_attachments: &[Some(wgpu::RenderPassColorAttachment {
				view: &view,
				resolve_target: None,
				ops: Operations {
					load: LoadOp::Clear(Color {
						r: 0.1,
						g: 0.4,
						b: 0.7,
						a: 1.0,
					}),
					store: true,
				},
			})],
			depth_stencil_attachment: None,
		});

		pass.set_pipeline(pipeline);
		pass.set_bind_group(0, &bind_group, &[]);

		pass.set_vertex_buffer(0, vertex_buffer.slice(..));
		pass.set_index_buffer(index_buffer.slice(..), IndexFormat::Uint16);
		pass.draw_indexed(0..indices.len() as u32, 0, 0..1);
	}

	queue.submit(Some(encoder.finish()));

	output.present();
}
