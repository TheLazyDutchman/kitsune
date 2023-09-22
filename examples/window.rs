use std::error::Error;

use kitsune_ui::{
	widget::{Column, Widget},
	window::Window,
};
use winit::event::{ElementState, KeyboardInput, WindowEvent};

struct Input {
	value: String,
}

impl Input {
	fn new() -> Self {
		Self {
			value: String::new(),
		}
	}
}

impl Widget for Input {
	type Renderable = <String as Widget>::Renderable;

	fn get_renderable(
		&mut self,
		context: &mut kitsune_ui::context::Context<kitsune_ui::widget::WidgetContext>,
		view: kitsune_ui::view::View,
	) -> Self::Renderable {
		self.value
			.get_renderable(context, view)
	}

	fn handle(&mut self, event: &WindowEvent) {
		if let WindowEvent::KeyboardInput {
			input:
				KeyboardInput {
					state: ElementState::Pressed,
					virtual_keycode,
					..
				},
			..
		} = event
		{
			use winit::event::VirtualKeyCode as C;
			if let Some(val) = virtual_keycode.and_then(|x| {
				Some(match x {
					C::A => 'a',
					C::B => 'b',
					C::C => 'c',
					C::D => 'd',
					C::E => 'e',
					C::F => 'f',
					C::G => 'g',
					C::H => 'h',
					C::I => 'i',
					C::J => 'j',
					C::K => 'k',
					C::L => 'l',
					C::M => 'm',
					C::N => 'n',
					C::O => 'o',
					C::P => 'p',
					C::Q => 'q',
					C::R => 'r',
					C::S => 's',
					C::T => 't',
					C::U => 'u',
					C::V => 'v',
					C::W => 'w',
					C::X => 'x',
					C::Y => 'y',
					C::Z => 'z',
					_ => None?,
				})
			}) {
				self.value.push(val);
			}
		}
	}

	fn width_hint(
		&self,
		context: &kitsune_ui::context::Context<kitsune_ui::widget::WidgetContext>,
	) -> kitsune_ui::view::SizeHint {
		self.value.width_hint(context)
	}

	fn height_hint(
		&self,
		context: &kitsune_ui::context::Context<kitsune_ui::widget::WidgetContext>,
	) -> kitsune_ui::view::SizeHint {
		self.value
			.height_hint(context)
	}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let widget = Column::new(vec![Input::new(), Input::new()]);

	let window = Window::new(widget).await?;

	window.run();
}
