use core::time::Duration;

use alloc::{boxed::Box, string::String};

use crate::{
    artboard::Artboard,
    instantiate::{Handle, Instantiate},
    linear_animation::{LinearAnimation, Loop},
    renderer::Renderer,
    state_machine::StateMachine,
};

pub(crate) fn transform(x: f32, y: f32, t: &[f32; 6]) -> [f32; 2] {
    [t[0] * x + t[2] * y + t[4], t[1] * x + t[3] * y + t[5]]
}

#[derive(Clone, Debug)]
pub struct Viewport {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) inverse_view_transform: [f32; 6],
}

impl Viewport {
    #[inline]
    pub fn width(&self) -> u32 {
        self.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.height
    }

    #[inline]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

impl Default for Viewport {
    #[inline]
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            inverse_view_transform: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

pub trait Scene<R: Renderer>: Send + Sync {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn name(&self) -> String;
    fn r#loop(&self) -> Loop;
    fn is_translucent(&self) -> bool;
    fn duration(&self) -> Option<Duration>;
    fn pointer_down(&mut self, x: f32, y: f32, viewport: &Viewport);
    fn pointer_move(&mut self, x: f32, y: f32, viewport: &Viewport);
    fn pointer_up(&mut self, x: f32, y: f32, viewport: &Viewport);
    fn advance_and_apply(&mut self, elapsed: Duration) -> bool;
    fn draw(&self, renderer: &mut R);
    fn advance_and_maybe_draw(
        &mut self,
        renderer: &mut R,
        elapsed: Duration,
        viewport: &mut Viewport,
    ) -> bool;
    fn as_any(&self) -> &dyn ::core::any::Any;
}

macro_rules! impl_scene {
    ( $type:tt ) => {
        impl<R: Renderer> crate::scene::Scene<R> for $type<R> {
            fn as_any(&self) -> &dyn ::core::any::Any {
                self
            }

            #[inline]
            fn width(&self) -> f32 {
                unsafe { crate::ffi::rive_rs_scene_width(self.raw_scene()) }
            }

            #[inline]
            fn height(&self) -> f32 {
                unsafe { crate::ffi::rive_rs_scene_height(self.raw_scene()) }
            }

            #[inline]
            fn name(&self) -> ::alloc::string::String {
                let mut name = ::alloc::string::String::new();

                unsafe {
                    crate::ffi::rive_rs_scene_name(
                        self.raw_scene(),
                        &mut name as *mut ::alloc::string::String,
                    );
                }

                name
            }

            #[inline]
            fn r#loop(&self) -> crate::linear_animation::Loop {
                unsafe { crate::ffi::rive_rs_scene_loop(self.raw_scene()) }
            }

            #[inline]
            fn is_translucent(&self) -> bool {
                unsafe { crate::ffi::rive_rs_scene_is_translucent(self.raw_scene()) }
            }

            #[inline]
            fn duration(&self) -> Option<::core::time::Duration> {
                ::core::time::Duration::try_from_secs_f32(unsafe {
                    crate::ffi::rive_rs_scene_duration(self.raw_scene())
                })
                .ok()
            }

            #[inline]
            fn pointer_down(&mut self, x: f32, y: f32, viewport: &crate::scene::Viewport) {
                let [x, y] = crate::scene::transform(x, y, &viewport.inverse_view_transform);
                unsafe {
                    crate::ffi::rive_rs_scene_pointer_down(self.raw_scene(), x, y);
                }
            }

            #[inline]
            fn pointer_move(&mut self, x: f32, y: f32, viewport: &crate::scene::Viewport) {
                let [x, y] = crate::scene::transform(x, y, &viewport.inverse_view_transform);
                unsafe {
                    crate::ffi::rive_rs_scene_pointer_move(self.raw_scene(), x, y);
                }
            }

            #[inline]
            fn pointer_up(&mut self, x: f32, y: f32, viewport: &crate::scene::Viewport) {
                let [x, y] = crate::scene::transform(x, y, &viewport.inverse_view_transform);
                unsafe {
                    crate::ffi::rive_rs_scene_pointer_up(self.raw_scene(), x, y);
                }
            }

            #[inline]
            fn advance_and_apply(&mut self, elapsed: ::core::time::Duration) -> bool {
                unsafe {
                    crate::ffi::rive_rs_scene_advance_and_apply(
                        self.raw_scene(),
                        elapsed.as_secs_f32(),
                    )
                }
            }

            #[inline]
            fn draw(&self, renderer: &mut R) {
                unsafe {
                    crate::ffi::rive_rs_scene_draw(
                        self.raw_scene(),
                        renderer as *mut R as *mut (),
                        crate::ffi::RendererEntries::<R>::ENTRIES
                            as *const crate::ffi::RendererEntries<R> as *const (),
                    );
                }
            }

            #[inline]
            fn advance_and_maybe_draw(
                &mut self,
                renderer: &mut R,
                elapsed: ::core::time::Duration,
                viewport: &mut crate::scene::Viewport,
            ) -> bool {
                let mut view_transform = [0.0; 6];
                let mut inverse_view_transform = [0.0; 6];

                unsafe {
                    crate::ffi::rive_rs_artboard_instance_transforms(
                        self.raw_artboard(),
                        viewport.width,
                        viewport.height,
                        view_transform.as_mut_ptr(),
                        inverse_view_transform.as_mut_ptr(),
                    );
                }

                viewport.inverse_view_transform = inverse_view_transform;

                if !self.advance_and_apply(elapsed) {
                    return false;
                }

                renderer.state_push();
                renderer.transform(&view_transform);

                self.draw(renderer);

                renderer.state_pop();

                true
            }
        }
    };
}

pub(crate) use impl_scene;

impl<R: Renderer> Instantiate for Box<dyn Scene<R>> {
    type From = Artboard<R>;

    fn instantiate(from: &Self::From, handle: Handle) -> Option<Self> {
        StateMachine::instantiate(from, handle.clone())
            .map(|sm| Box::new(sm) as Box<dyn Scene<R>>)
            .or_else(|| {
                LinearAnimation::instantiate(from, handle)
                    .map(|la| Box::new(la) as Box<dyn Scene<R>>)
            })
    }
}
