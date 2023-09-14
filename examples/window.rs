use wgpu::{
	Color, CompositeAlphaMode, Device, LoadOp, Operations, PresentMode, Queue,
	RenderPassDescriptor, Surface, SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::{
	event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
	event_loop::EventLoop,
	window::Window,
};

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
			draw(&surface, &device, &queue);
		}
		_ => {}
	});
}

fn draw(surface: &Surface, device: &Device, queue: &Queue) {
	let output = surface
		.get_current_texture()
		.unwrap();
	let view = output
		.texture
		.create_view(&Default::default());

	let mut encoder = device.create_command_encoder(&Default::default());
	encoder.begin_render_pass(&RenderPassDescriptor {
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

	queue.submit(Some(encoder.finish()));

	output.present();
}
