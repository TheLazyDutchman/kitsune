use std::error::Error;

use kitsune_ui::{widget::Bordered, window::Window};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let widget = Bordered::new("Hello, World".to_string(), 4);
	let window = Window::new(widget).await?;

	window.run();
}
