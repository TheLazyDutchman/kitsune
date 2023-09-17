use std::error::Error;

use kitsune_ui::window::Window;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let window = Window::new("Hello, World!".to_string()).await?;

	window.run();
}
