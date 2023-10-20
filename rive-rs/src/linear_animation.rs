use alloc::sync::Arc;
use core::{fmt, marker::PhantomData, ptr::NonNull, time::Duration};

use crate::{
    artboard::{Artboard, ArtboardInner},
    ffi,
    instantiate::{Handle, Instantiate},
    renderer::Renderer,
    scene::impl_scene,
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Direction {
    Forwards,
    Backwards,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Loop {
    /// Play until the duration or end of work area of the animation.
    OneShot = 0,
    /// Play until the duration or end of work area of the animation and
    /// then go back to the start (0 seconds).
    Loop = 1,
    /// Play to the end of the duration/work area and then play back.
    PingPong = 2,
}

pub struct LinearAnimation<R: Renderer> {
    artboard: Arc<ArtboardInner>,
    raw_linear_animation: *mut ffi::LinearAnimation,
    _phantom: PhantomData<R>,
}

impl<R: Renderer> Instantiate for LinearAnimation<R> {
    type From = Artboard<R>;

    #[inline]
    fn instantiate(artboard: &Self::From, handle: Handle) -> Option<Self> {
        let mut raw_linear_animation: Option<NonNull<ffi::LinearAnimation>> = None;

        match handle {
            Handle::Default => unsafe {
                ffi::rive_rs_instantiate_linear_animation(
                    artboard.as_inner().raw_artboard,
                    None,
                    &mut raw_linear_animation,
                )
            },
            Handle::Index(ref index) => unsafe {
                ffi::rive_rs_instantiate_linear_animation(
                    artboard.as_inner().raw_artboard,
                    Some(index.into()),
                    &mut raw_linear_animation,
                )
            },
            Handle::Name(name) => unsafe {
                ffi::rive_rs_instantiate_linear_animation_by_name(
                    artboard.as_inner().raw_artboard,
                    name.as_ptr(),
                    name.len(),
                    &mut raw_linear_animation,
                )
            },
        }

        raw_linear_animation.map(|raw_linear_animation| LinearAnimation {
            artboard: artboard.as_inner().clone(),
            raw_linear_animation: raw_linear_animation.as_ptr(),
            _phantom: PhantomData,
        })
    }
}

impl<R: Renderer> LinearAnimation<R> {
    fn raw_artboard(&self) -> *mut ffi::Artboard {
        self.artboard.raw_artboard
    }

    pub fn artboard(&self) -> Artboard<R> {
        Artboard::from_inner(self.artboard.clone())
    }

    fn raw_scene(&self) -> *mut ffi::Scene {
        self.raw_linear_animation as *mut ffi::Scene
    }

    pub fn time(&self) -> Duration {
        Duration::from_secs_f32(unsafe {
            ffi::rive_rs_linear_animation_time(self.raw_linear_animation)
        })
    }

    pub fn set_time(&mut self, time: Duration) {
        unsafe {
            ffi::rive_rs_linear_animation_set_time(self.raw_linear_animation, time.as_secs_f32());
        }
    }

    pub fn direction(&self) -> Direction {
        match unsafe { ffi::rive_rs_linear_animation_is_forwards(self.raw_linear_animation) } {
            true => Direction::Forwards,
            false => Direction::Backwards,
        }
    }

    pub fn set_direction(&mut self, direction: Direction) {
        unsafe {
            ffi::rive_rs_linear_animation_set_is_forwards(
                self.raw_linear_animation,
                direction == Direction::Forwards,
            );
        }
    }

    pub fn advance(&mut self, elapsed: Duration) -> bool {
        unsafe {
            ffi::rive_rs_linear_animation_advance(self.raw_linear_animation, elapsed.as_secs_f32())
        }
    }

    pub fn apply(&mut self, mix: f32) {
        unsafe { ffi::rive_rs_linear_animation_apply(self.raw_linear_animation, mix) }
    }

    pub fn did_loop(&self) -> bool {
        unsafe { ffi::rive_rs_linear_animation_did_loop(self.raw_linear_animation) }
    }

    pub fn set_loop(&mut self, r#loop: Loop) {
        unsafe {
            ffi::rive_rs_linear_animation_set_loop(self.raw_linear_animation, r#loop);
        }
    }

    pub fn is_done(&self) -> bool {
        unsafe { ffi::rive_rs_linear_animation_is_done(self.raw_linear_animation) }
    }
}

impl<R: Renderer> fmt::Debug for LinearAnimation<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LinearAnimation").finish()
    }
}

impl<R: Renderer> Drop for LinearAnimation<R> {
    fn drop(&mut self) {
        unsafe {
            ffi::rive_rs_scene_release(self.raw_scene());
        }
    }
}

unsafe impl<R: Renderer> Send for LinearAnimation<R> {}
unsafe impl<R: Renderer> Sync for LinearAnimation<R> {}

impl_scene!(LinearAnimation);
