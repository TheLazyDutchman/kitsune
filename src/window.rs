use inner::WindowInner;
use thiserror::Error;
use winit::{
	event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
	event_loop::EventLoop,
};

use crate::widget::Widget;

type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Error when creating a window: `{0}`")]
	OsError(#[from] winit::error::OsError),

	#[error("Error when creating a draw surface: `{0}`")]
	CreateSurfaceError(#[from] wgpu::CreateSurfaceError),

	#[error("Could not find an adapter")]
	AdapterNotFound,

	#[error("Could not requeset a device: `{0}`")]
	RequestDeviceError(#[from] wgpu::RequestDeviceError),

	#[error("The supplied font file is not of a valid TTF format")]
	InvalidFont(#[from] ab_glyph::InvalidFont),

	#[error("Could not get the current texture of the draw surface")]
	SurfaceError(#[from] wgpu::SurfaceError),
}

mod inner {
	use ab_glyph::FontRef;
	use winit::{
		event_loop::EventLoop,
		window::{Window, WindowId},
	};

	use super::{Error, Result};
	use crate::{
		context::Context,
		render::{Render, RenderContext, Vertex},
		text::Font,
		view::GlobalView,
		widget::{Widget, WidgetContext},
	};

	pub struct WindowInner<T> {
		window: Window,
		device: wgpu::Device,
		queue: wgpu::Queue,
		config: wgpu::SurfaceConfiguration,
		surface: wgpu::Surface,
		pipeline: wgpu::RenderPipeline,
		global_view: GlobalView,
		font: Font,
		size: winit::dpi::PhysicalSize<u32>,
		widget: T,
		bind_group_layout: wgpu::BindGroupLayout,
		sampler: wgpu::Sampler,
	}

	impl<T: Widget> WindowInner<T> {
		pub async fn new(event_loop: &EventLoop<()>, widget: T) -> Result<Self> {
			let window = Window::new(event_loop)?;

			let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
				backends: wgpu::Backends::all(),
				..Default::default()
			});

			let surface = unsafe { instance.create_surface(&window) }?;

			let adapter = instance
				.request_adapter(&wgpu::RequestAdapterOptions {
					power_preference: wgpu::PowerPreference::default(),
					force_fallback_adapter: false,
					compatible_surface: Some(&surface),
				})
				.await
				.ok_or(Error::AdapterNotFound)?;

			let size = window.inner_size();

			let surface_caps = surface.get_capabilities(&adapter);
			let surface_format = surface_caps
				.formats
				.iter()
				.copied()
				.find(|f| f.is_srgb())
				.unwrap_or(surface_caps.formats[0]);

			let config = wgpu::SurfaceConfiguration {
				usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
				format: surface_format,
				width: size.width,
				height: size.height,
				present_mode: surface_caps.present_modes[0],
				alpha_mode: surface_caps.alpha_modes[0],
				view_formats: vec![],
			};

			let (device, queue) = adapter
				.request_device(
					&wgpu::DeviceDescriptor {
						features: wgpu::Features::empty(),
						//TODO: for wasm32, this should be `downlevel_webg12_defaults`
						limits: wgpu::Limits::default(),
						label: None,
					},
					None,
				)
				.await?;

			surface.configure(&device, &config);

			let bind_group_layout =
				device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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

			let font = Font::new(FontRef::try_from_slice(include_bytes!(
				"../res/Roboto/Roboto-Medium.ttf"
			))?);

			let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

			let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
				label: Some("Render Pipeline Layout"),
				bind_group_layouts: &[&bind_group_layout],
				push_constant_ranges: &[],
			});

			let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
				label: Some("Render Pipeline"),
				layout: Some(&layout),
				vertex: wgpu::VertexState {
					module: &shader,
					entry_point: "vs_main",
					buffers: &[Vertex::layout()],
				},
				fragment: Some(wgpu::FragmentState {
					module: &shader,
					entry_point: "fs_main",
					targets: &[Some(wgpu::ColorTargetState {
						format: config.format,
						blend: Some(wgpu::BlendState::ALPHA_BLENDING),
						write_mask: wgpu::ColorWrites::ALL,
					})],
				}),
				primitive: wgpu::PrimitiveState {
					topology: wgpu::PrimitiveTopology::TriangleList,
					strip_index_format: None,
					front_face: wgpu::FrontFace::Ccw,
					cull_mode: Some(wgpu::Face::Back),
					polygon_mode: wgpu::PolygonMode::Fill,
					unclipped_depth: false,
					conservative: false,
				},
				depth_stencil: None,
				multisample: wgpu::MultisampleState {
					count: 1,
					mask: !0,
					alpha_to_coverage_enabled: false,
				},
				multiview: None,
			});

			let global_view = GlobalView::new(size);

			Ok(Self {
				window,
				size,
				device,
				queue,
				config,
				surface,
				font,
				pipeline,
				global_view,
				widget,
				sampler,
				bind_group_layout,
			})
		}

		pub fn id(&self) -> WindowId {
			self.window.id()
		}

		pub fn request_redraw(&self) {
			self.window.request_redraw()
		}

		pub fn resize(&mut self, inner_size: winit::dpi::PhysicalSize<u32>) {
			self.size = inner_size;
			self.config.width = inner_size.width;
			self.config.height = inner_size.height;

			self.surface
				.configure(&self.device, &self.config);

			self.global_view = GlobalView::new(inner_size);

			self.widget.resize(inner_size);
		}

		pub fn draw(&mut self) -> Result<()> {
			let output = self
				.surface
				.get_current_texture()?;
			let texture_view = output
				.texture
				.create_view(&Default::default());

			let mut encoder = self
				.device
				.create_command_encoder(&Default::default());

			let view = self
				.global_view
				.view(self.size, winit::dpi::PhysicalPosition::new(0, 0));

			let mut context = Context::new(WidgetContext::new(
				&self.font,
				&self.device,
				&self.queue,
				&self.config,
				&self.sampler,
				&self.bind_group_layout,
			));

			let widget = self
				.widget
				.get_renderable(&mut context, view);

			{
				let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
					label: Some("Render Pass"),
					color_attachments: &[Some(wgpu::RenderPassColorAttachment {
						view: &texture_view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(wgpu::Color {
								r: 0.1,
								g: 0.5,
								b: 0.9,
								a: 1.0,
							}),
							store: true,
						},
					})],
					depth_stencil_attachment: None,
				});

				pass.set_pipeline(&self.pipeline);

				let mut context = Context::new(RenderContext::new(pass));

				widget.render(&mut context);
			}

			self.queue
				.submit(Some(encoder.finish()));

			output.present();

			Ok(())
		}
	}
}

pub struct Window<T> {
	inner: WindowInner<T>,
	event_loop: EventLoop<()>,
}

impl<T: Widget> Window<T> {
	pub async fn new(widget: T) -> Result<Self> {
		let event_loop = EventLoop::new();
		let inner = WindowInner::new(&event_loop, widget).await?;

		Ok(Self { event_loop, inner })
	}

	pub fn run(mut self) -> !
	where
		T: 'static,
	{
		self.event_loop
			.run(move |event, _, control_flow| match event {
				Event::WindowEvent { window_id, event } if self.inner.id() == window_id => {
					match event {
						WindowEvent::CloseRequested
						| WindowEvent::KeyboardInput {
							input:
								KeyboardInput {
									state: ElementState::Pressed,
									virtual_keycode: Some(VirtualKeyCode::Escape),
									..
								},
							..
						} => control_flow.set_exit(),
						WindowEvent::Resized(new_size)
						| WindowEvent::ScaleFactorChanged {
							new_inner_size: &mut new_size,
							..
						} => self.inner.resize(new_size),
						_ => {}
					}
				}
				Event::MainEventsCleared => self.inner.request_redraw(),
				Event::RedrawRequested(window_id) if self.inner.id() == window_id => {
					let result = self.inner.draw();
					if result.is_err() {
						control_flow.set_exit();
					}
				}
				_ => {}
			});
	}
}
