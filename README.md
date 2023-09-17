# KITSUNE UI

> ## ❗ disclaimer
>
> This is just a proof of concept for now, and it is by no means meant to be used yet.

This is a retained mode User Interface library.

## Features

- [ ] Dont compile things that haven't changed.
- [ ] Add macro to implement `widget` for user types.
- [ ] Add ways to layout values, and control their size.
- [ ] Find a way to do user input.
- [ ] Maybe find a way to changed state based on Non-user events.

## Getting Started

Using the `window` feature, it is very easy to draw a widget in a `winit` window.

```rust
use kitsune_ui::window::{Window, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let widget = 'a';

    let window = Window::new(widget).await?;

    window.run();

    Ok(())
}
```

Here the `widget` could be any value that implements the `kitsune_ui::widget::Widget` trait.
