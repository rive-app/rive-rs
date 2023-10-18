# Vello back-end for Rive

Small application for viewing `.riv` files rendered with Vello. It uses [winit]
for creating the window and [image] to decode images.

## Usage

Drop any `.riv` file into the window to open it. Scroll to control the size of
the grid of copies.

## Caveats

The current implementation is a work-in-progress and might exhibit artifacts or
render incorrectly.

Only tested on macOS for the time being.

[winit]: https://github.com/rust-windowing/winit
[image]: https://github.com/image-rs/image
