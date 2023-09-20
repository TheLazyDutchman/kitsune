use std::error::Error;

use kitsune_ui::{
	widget::{Bordered, Widget},
	window::Window,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let widget = Bordered::new(
		"Hello, World"
			.to_string()
			.cached(),
		4,
	);
	let window = Window::new(widget).await?;

	window.run();
}
