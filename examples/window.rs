use std::error::Error;

use kitsune_ui::{widget::Widget, window::Window};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let widget = "Hello, World"
		.to_string()
		.bordered(4)
		.cached();

	let window = Window::new(widget).await?;

	window.run();
}
