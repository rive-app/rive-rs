use anyhow::Result;
use rive_rs::{Artboard, File, Handle, Instantiate, Viewport};
use std::fs;
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use vello::kurbo::Vec2;
use vello::peniko::color::palette;
use vello::util::{RenderContext, RenderSurface};
use vello::{AaConfig, Renderer, RendererOptions};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::Window;

use vello::wgpu;


#[derive(Debug)]
enum RenderState<'s> {
    Active {
        surface: Box<RenderSurface<'s>>,
        window: Arc<Window>,
    },
    Suspended(Option<Arc<Window>>),
}

struct RiveViewerApp<'s> {
    // Global Vello Render Context
    context: RenderContext,

    // Wgpu renderers per device
    renderers: Vec<Option<Renderer>>,

    // Winit Render State
    state: RenderState<'s>,

    // Rive Scene
    rive_scene: Option<Box<dyn rive_rs::Scene>>,

    // Rive Viewport
    viewport: Viewport,

    mouse_pos: Vec2,

    frame_start_time: Instant,

    prev_renderer: rive_rs::Renderer,
}

impl ApplicationHandler for RiveViewerApp<'_> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let RenderState::Suspended(cached_window) = &mut self.state else {
            return;
        };

        let window = cached_window
            .take()
            .unwrap_or_else(|| create_winit_window(event_loop));

        let size = window.inner_size();
        let surface_future = self.context.create_surface(
            window.clone(),
            size.width,
            size.height,
            wgpu::PresentMode::AutoVsync,
        );
        let surface = pollster::block_on(surface_future).expect("Error creating surface");

        self.renderers
            .resize_with(self.context.devices.len(), || None);
        self.renderers[surface.dev_id]
            .get_or_insert_with(|| create_vello_renderer(&self.context, &surface));

        self.state = RenderState::Active {
            surface: Box::new(surface),
            window,
        };
    }

    fn suspended(&mut self, _event_loop: &ActiveEventLoop) {
        if let RenderState::Active { window, .. } = &self.state {
            self.state = RenderState::Suspended(Some(window.clone()));
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let surface = match &mut self.state {
            RenderState::Active { surface, window } if window.id() == window_id => surface,
            _ => return,
        };

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                self.viewport.resize(size.width, size.height);

                self.context
                    .resize_surface(surface, size.width, size.height);

                if let RenderState::Active { surface: _, window } = &mut self.state {
                    window.request_redraw();
                }
            }

            WindowEvent::Moved(_) => {
                if let RenderState::Active { surface: _, window } = &mut self.state {
                    window.request_redraw();
                }
            }

            WindowEvent::DroppedFile(path) => {
                self.rive_scene = Some({
                    let file = File::new(&fs::read(path).unwrap()).unwrap();
                    let artboard = Artboard::instantiate(&file, Handle::Default).unwrap();

                    Box::<dyn rive_rs::Scene>::instantiate(&artboard, Handle::Default).unwrap()
                });
                if let RenderState::Active { surface: _, window } = &mut self.state {
                    window.request_redraw();
                }
            }
            WindowEvent::MouseInput {
                state,
                button: MouseButton::Left,
                ..
            } => {
                if let Some(scene) = &mut self.rive_scene {
                    match state {
                        ElementState::Pressed => scene.pointer_down(
                            self.mouse_pos.x as f32,
                            self.mouse_pos.y as f32,
                            &self.viewport,
                        ),
                        ElementState::Released => scene.pointer_up(
                            self.mouse_pos.x as f32,
                            self.mouse_pos.y as f32,
                            &self.viewport,
                        ),
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.mouse_pos = Vec2::new(position.x, position.y);
                if let Some(scene) = &mut self.rive_scene {
                    scene.pointer_move(
                        self.mouse_pos.x as f32,
                        self.mouse_pos.y as f32,
                        &self.viewport,
                    );
                }
            }

            WindowEvent::RedrawRequested => {
                let mut rive_renderer = rive_rs::Renderer::default();

                let elapsed = &self.frame_start_time.elapsed();

                self.frame_start_time = Instant::now();

                let RenderState::Active { surface, window } = &mut self.state else {
                    return;
                };

                window.set_title(&format!(
                    "Rive Demo | {:.2}ms",
                    elapsed.as_secs_f64() * 1000.0
                ));

                if let Some(scene) = &mut self.rive_scene {
                    if scene.advance_and_maybe_draw(
                        &mut rive_renderer,
                        *elapsed,
                        &mut self.viewport,
                    ) {
                        self.prev_renderer = rive_renderer;
                    } else {
                        // The rive scene has nothing new to render, don't update the rendered texture.
                        window.request_redraw();
                        sleep(Duration::from_millis(2));
                        return;
                    }
                }

                let width = surface.config.width;
                let height = surface.config.height;

                let device_handle = &self.context.devices[surface.dev_id];

                // Render the vello scene to surface.target_view.
                self.renderers[surface.dev_id]
                    .as_mut()
                    .unwrap()
                    .render_to_texture(
                        &device_handle.device,
                        &device_handle.queue,
                        self.prev_renderer.scene(),
                        &surface.target_view,
                        &vello::RenderParams {
                            base_color: palette::css::GRAY, // Background color
                            width,
                            height,
                            antialiasing_method: AaConfig::Msaa16,
                        },
                    )
                    .expect("failed to render to surface");

                // Grab the texture for the window.
                let surface_texture = surface
                    .surface
                    .get_current_texture()
                    .expect("failed to get surface texture");

                // Copy surface.target_view to surface_texture. (This handles any formatting differences )
                let mut encoder =
                    device_handle
                        .device
                        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("Surface Blit"),
                        });
                surface.blitter.copy(
                    &device_handle.device,
                    &mut encoder,
                    &surface.target_view,
                    &surface_texture
                        .texture
                        .create_view(&wgpu::TextureViewDescriptor::default()),
                );
                device_handle.queue.submit([encoder.finish()]);

                // Present texture on screen.
                surface_texture.present();

                device_handle.device.poll(wgpu::Maintain::Poll);

                window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() -> Result<()> {
    let mut app = RiveViewerApp {
        context: RenderContext::new(),
        renderers: vec![],
        state: RenderState::Suspended(None),
        rive_scene: None,
        viewport: Viewport::default(),
        mouse_pos: Vec2::default(),
        frame_start_time: Instant::now(),
        prev_renderer: rive_rs::Renderer::default(),
    };

    let event_loop = EventLoop::new()?;
    event_loop
        .run_app(&mut app)
        .expect("Couldn't run event loop");
    Ok(())
}

fn create_winit_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    let attr = Window::default_attributes()
        .with_inner_size(LogicalSize::new(1044, 800))
        .with_resizable(true)
        .with_title("Rive Demo | 0ms");
    Arc::new(event_loop.create_window(attr).unwrap())
}

fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface<'_>) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions::default(),
    )
    .expect("Couldn't create renderer")
}
