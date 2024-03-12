use std::{fs, num::NonZeroUsize, sync::Arc, time::Instant};

use rive_rs::{Artboard, File, Handle, Instantiate, Viewport};
use vello::{
    kurbo::{Affine, Rect, Vec2},
    peniko::{Color, Fill},
    util::{RenderContext, RenderSurface},
    Renderer, RendererOptions, Scene as VelloScene,
};
use winit::{
    dpi::LogicalSize,
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

// Simple struct to hold the state of the renderer
pub struct ActiveRenderState<'s> {
    // The fields MUST be in this order, so that the surface is dropped before the window
    surface: RenderSurface<'s>,
    window: Arc<Window>,
}

enum RenderState<'s> {
    Active(ActiveRenderState<'s>),
    // Cache a window so that it can be reused when the app is resumed after being suspended
    Suspended(Option<Arc<Window>>),
}

const INITIAL_WINDOW_SIZE: LogicalSize<u32> = LogicalSize::new(700, 700);
const FRAME_STATS_CAPACITY: usize = 30;
const SCROLL_FACTOR_THRESHOLD: f64 = 100.0;

fn main() {
    let mut viewport = Viewport::default();
    let mut scene: Option<Box<dyn rive_rs::Scene>> = None;

    // An array of renderers, one per wgpu device
    let mut renderers: Vec<Option<Renderer>> = vec![];
    let mut render_cx = RenderContext::new().unwrap();
    // let mut render_state: Option<RenderState> = None;
    let mut render_state = RenderState::Suspended(None);

    let mut mouse_pos = Vec2::default();
    let mut scroll_delta = 0.0;
    let mut frame_start_time = Instant::now();
    let mut stats = Vec::with_capacity(FRAME_STATS_CAPACITY);

    let event_loop = EventLoop::new().unwrap();
    let _ = event_loop.run(move |event, event_loop| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } => {
            let render_state = match &mut render_state {
                RenderState::Active(state) if state.window.id() == window_id => state,
                _ => return,
            };

            match event {
                WindowEvent::CloseRequested => event_loop.exit(),
                WindowEvent::Resized(size) => {
                    viewport.resize(size.width, size.height);

                    render_cx.resize_surface(&mut render_state.surface, size.width, size.height);
                    render_state.window.request_redraw();
                }
                WindowEvent::MouseInput {
                    state,
                    button: MouseButton::Left,
                    ..
                } => {
                    if let Some(scene) = &mut scene {
                        match state {
                            ElementState::Pressed => scene.pointer_down(
                                mouse_pos.x as f32,
                                mouse_pos.y as f32,
                                &viewport,
                            ),
                            ElementState::Released => {
                                scene.pointer_up(mouse_pos.x as f32, mouse_pos.y as f32, &viewport)
                            }
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    mouse_pos = Vec2::new(position.x, position.y);
                    if let Some(scene) = &mut scene {
                        scene.pointer_move(mouse_pos.x as f32, mouse_pos.y as f32, &viewport);
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, lines_y) => {
                        scroll_delta = (scroll_delta
                            - (*lines_y as f64).signum() * SCROLL_FACTOR_THRESHOLD)
                            .max(0.0);
                    }
                    winit::event::MouseScrollDelta::PixelDelta(pixels) => {
                        scroll_delta = (scroll_delta - pixels.y).max(0.0);
                    }
                },
                WindowEvent::DroppedFile(path) => {
                    scene = Some({
                        let file = File::new(&fs::read(path).unwrap()).unwrap();
                        let artboard = Artboard::instantiate(&file, Handle::Default).unwrap();

                        Box::<dyn rive_rs::Scene>::instantiate(&artboard, Handle::Default).unwrap()
                    });
                }
                WindowEvent::RedrawRequested => {
                    let mut rive_renderer = rive_rs::Renderer::default();
                    let factor = (scroll_delta / SCROLL_FACTOR_THRESHOLD).max(1.0) as u32;

                    let elapsed = &frame_start_time.elapsed();
                    stats.push(elapsed.as_secs_f64());

                    frame_start_time = Instant::now();
                    let surface = &render_state.surface;

                    let width = render_state.surface.config.width;
                    let height = render_state.surface.config.height;
                    let device_handle = &render_cx.devices[render_state.surface.dev_id];

                    let render_params = vello::RenderParams {
                        base_color: Color::DIM_GRAY,
                        width,
                        height,
                        antialiasing_method: vello::AaConfig::Msaa16,
                    };

                    let surface_texture = render_state
                        .surface
                        .surface
                        .get_current_texture()
                        .expect("failed to get surface texture");

                    let mut vello_scene = VelloScene::default();

                    if let Some(scene) = &mut scene {
                        scene.advance_and_maybe_draw(&mut rive_renderer, *elapsed, &mut viewport);

                        for i in 0..factor.pow(2) {
                            vello_scene.append(
                                rive_renderer.scene(),
                                Some(
                                    Affine::default()
                                        .then_scale(1.0 / factor as f64)
                                        .then_translate(Vec2::new(
                                            (i % factor) as f64 * width as f64 / factor as f64,
                                            (i / factor) as f64 * height as f64 / factor as f64,
                                        )),
                                ),
                            );
                        }
                    } else {
                        // Vello doesn't draw base color when there is no geometry.
                        vello_scene.fill(
                            Fill::NonZero,
                            Affine::IDENTITY,
                            Color::TRANSPARENT,
                            None,
                            &Rect::new(0.0, 0.0, 0.0, 0.0),
                        );
                    }

                    if !vello_scene.encoding().is_empty() {
                        vello::block_on_wgpu(
                            &device_handle.device,
                            renderers[surface.dev_id]
                                .as_mut()
                                .unwrap()
                                .render_to_surface_async(
                                    &device_handle.device,
                                    &device_handle.queue,
                                    &vello_scene,
                                    &surface_texture,
                                    &render_params,
                                ),
                        )
                        .expect("failed to render to surface");
                    }

                    surface_texture.present();
                    device_handle.device.poll(wgpu::Maintain::Poll);
                }
                _ => {}
            }
        }
        Event::Suspended => {
            if let RenderState::Active(state) = &render_state {
                render_state = RenderState::Suspended(Some(state.window.clone()));
            }
            event_loop.set_control_flow(ControlFlow::Wait);
        }
        Event::Resumed => {
            let RenderState::Suspended(cached_window) = &mut render_state else {
                return;
            };

            // Get the winit window cached in a previous Suspended event or else create a new window
            let window = cached_window
                .take()
                .unwrap_or_else(|| create_winit_window(event_loop));

            // Create a vello Surface
            let size = window.inner_size();
            let surface_future = render_cx.create_surface(
                window.clone(),
                size.width,
                size.height,
                wgpu::PresentMode::AutoNoVsync,
            );
            let surface = pollster::block_on(surface_future).expect("Error creating surface");

            // Create a vello Renderer for the surface (using its device id)
            renderers.resize_with(render_cx.devices.len(), || None);
            renderers[surface.dev_id]
                .get_or_insert_with(|| create_vello_renderer(&render_cx, &surface));

            // Save the Window and Surface to a state variable
            render_state = RenderState::Active(ActiveRenderState { window, surface });

            event_loop.set_control_flow(ControlFlow::Poll);
        }

        _ => {}
    });
}

/// Helper function that creates a Winit window and returns it (wrapped in an Arc for sharing between threads)
fn create_winit_window(event_loop: &winit::event_loop::EventLoopWindowTarget<()>) -> Arc<Window> {
    Arc::new(
        WindowBuilder::new()
            .with_inner_size(LogicalSize::new(1044, 800))
            .with_resizable(true)
            .with_title("Rive on Vello demo")
            .build(event_loop)
            .unwrap(),
    )
}

/// Helper function that creates a vello Renderer for a given RenderContext and Surface
fn create_vello_renderer(render_cx: &RenderContext, surface: &RenderSurface) -> Renderer {
    Renderer::new(
        &render_cx.devices[surface.dev_id].device,
        RendererOptions {
            surface_format: Some(surface.format),
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: NonZeroUsize::new(1),
        },
    )
    .expect("Could create renderer")
}
