use alloc::sync::Arc;
use core::{fmt, marker::PhantomData, ptr::NonNull};

use crate::{
    artboard::{Artboard, ArtboardInner},
    ffi,
    instantiate::{Handle, Instantiate},
    renderer::Renderer,
    scene::impl_scene,
};

mod events;
mod inputs;

pub use self::{
    events::{Event, EventIter, Property},
    inputs::{Bool, InputIter, Number, Trigger},
};

pub struct StateMachine<R: Renderer> {
    artboard: Arc<ArtboardInner>,
    raw_state_machine: *mut ffi::StateMachine,
    _phantom: PhantomData<R>,
}

impl<R: Renderer> Instantiate for StateMachine<R> {
    type From = Artboard<R>;

    #[inline]
    fn instantiate(artboard: &Self::From, handle: Handle) -> Option<Self> {
        let mut raw_state_machine: Option<NonNull<ffi::StateMachine>> = None;

        match handle {
            Handle::Default => unsafe {
                ffi::rive_rs_instantiate_state_machine(
                    artboard.as_inner().raw_artboard,
                    None,
                    &mut raw_state_machine,
                )
            },
            Handle::Index(ref index) => unsafe {
                ffi::rive_rs_instantiate_state_machine(
                    artboard.as_inner().raw_artboard,
                    Some(index.into()),
                    &mut raw_state_machine,
                )
            },
            Handle::Name(name) => unsafe {
                ffi::rive_rs_instantiate_state_machine_by_name(
                    artboard.as_inner().raw_artboard,
                    name.as_ptr(),
                    name.len(),
                    &mut raw_state_machine,
                )
            },
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

    pub fn artboart(&self) -> Artboard<R> {
        Artboard::from_inner(self.artboard.clone())
    }

    fn raw_scene(&self) -> *mut ffi::Scene {
        self.raw_state_machine as *mut ffi::Scene
    }

    #[inline]
    pub fn events(&self) -> EventIter {
        EventIter::new(events::RawStateMachine(self.raw_state_machine))
    }

    #[inline]
    pub fn inputs(&self) -> InputIter {
        InputIter::new(inputs::RawStateMachine(self.raw_state_machine))
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
