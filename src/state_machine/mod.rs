use alloc::sync::Arc;
use core::{fmt, marker::PhantomData, ptr::NonNull};

use crate::{
    artboard::{Artboard, ArtboardInner},
    ffi,
    instantiate::Instantiate,
    renderer::Renderer,
    scene::impl_scene,
};

mod event;
mod input;

pub use self::{
    event::{Event, EventIter, Property},
    input::{Bool, InputIter, Number, Trigger},
};

pub struct StateMachine<R: Renderer> {
    artboard: Arc<ArtboardInner>,
    raw_state_machine: *mut ffi::StateMachine,
    _phantom: PhantomData<R>,
}

impl<R: Renderer> Instantiate for StateMachine<R> {
    type From = Artboard<R>;

    #[inline]
    fn instantiate(artboard: &Self::From, index: Option<usize>) -> Option<Self> {
        let mut raw_state_machine: Option<NonNull<ffi::StateMachine>> = None;

        unsafe {
            ffi::rive_rs_instantiate_state_machine(
                artboard.as_inner().raw_artboard,
                index.as_ref().map(|val| val.into()),
                &mut raw_state_machine,
            );
        }

        raw_state_machine.map(|raw_state_machine| StateMachine {
            artboard: artboard.as_inner().clone(),
            raw_state_machine: raw_state_machine.as_ptr(),
            _phantom: PhantomData,
        })
    }
}

impl<R: Renderer> StateMachine<R> {
    fn raw_artboard(&self) -> *mut ffi::Artboard {
        self.artboard.raw_artboard
    }

    fn raw_scene(&self) -> *mut ffi::Scene {
        self.raw_state_machine as *mut ffi::Scene
    }

    #[inline]
    pub fn events(&self) -> EventIter {
        EventIter::new(event::RawStateMachine(self.raw_state_machine))
    }

    #[inline]
    pub fn inputs(&self) -> InputIter {
        InputIter::new(input::RawStateMachine(self.raw_state_machine))
    }

    #[inline]
    pub fn get_bool(&self, name: &str) -> Option<Bool> {
        unsafe {
            NonNull::new(ffi::rive_rs_state_machine_get_bool(
                self.raw_state_machine,
                name.as_ptr(),
                name.len(),
            ))
            .map(|ptr| Bool::new(ptr.as_ptr()))
        }
    }

    #[inline]
    pub fn get_number(&self, name: &str) -> Option<Number> {
        unsafe {
            NonNull::new(ffi::rive_rs_state_machine_get_number(
                self.raw_state_machine,
                name.as_ptr(),
                name.len(),
            ))
            .map(|ptr| Number::new(ptr.as_ptr()))
        }
    }

    #[inline]
    pub fn get_trigger(&self, name: &str) -> Option<Trigger> {
        unsafe {
            NonNull::new(ffi::rive_rs_state_machine_get_trigger(
                self.raw_state_machine,
                name.as_ptr(),
                name.len(),
            ))
            .map(|ptr| Trigger::new(ptr.as_ptr()))
        }
    }
}

impl<R: Renderer> fmt::Debug for StateMachine<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StateMachine").finish()
    }
}

impl<R: Renderer> Drop for StateMachine<R> {
    fn drop(&mut self) {
        unsafe {
            ffi::rive_rs_scene_release(self.raw_scene());
        }
    }
}

unsafe impl<R: Renderer> Send for StateMachine<R> {}
unsafe impl<R: Renderer> Sync for StateMachine<R> {}

impl_scene!(StateMachine);