#![cfg_attr(not(feature = "vello"), no_std)]

extern crate alloc;

mod artboard;
mod ffi;
mod file;
mod instantiate;
mod linear_animation;
pub mod path;
mod raw_iter;
pub mod renderer;
pub mod scene;
pub mod state_machine;
#[cfg(feature = "vello")]
pub mod vello;

pub use crate::{
    artboard::components,
    file::Error,
    instantiate::{Handle, Instantiate},
    linear_animation::{Direction, Loop},
    scene::Viewport,
};

#[cfg(not(feature = "vello"))]
pub use crate::{
    artboard::Artboard, file::File, linear_animation::LinearAnimation, scene::Scene,
    state_machine::StateMachine,
};

#[cfg(feature = "vello")]
pub type Artboard = artboard::Artboard<crate::vello::Renderer>;
#[cfg(feature = "vello")]
pub type File = file::File<crate::vello::Renderer>;
#[cfg(feature = "vello")]
pub type LinearAnimation = linear_animation::LinearAnimation<crate::vello::Renderer>;
#[cfg(feature = "vello")]
pub type StateMachine = state_machine::StateMachine<crate::vello::Renderer>;
#[cfg(feature = "vello")]
pub use crate::vello::Renderer;

#[cfg(feature = "vello")]
pub trait Scene: scene::Scene<crate::vello::Renderer> {}
#[cfg(feature = "vello")]
impl<T: scene::Scene<crate::vello::Renderer>> Scene for T {}

#[cfg(feature = "vello")]
impl Instantiate for Box<dyn Scene> {
    type From = Artboard;

    fn instantiate(from: &Self::From, handle: Handle) -> Option<Self> {
        StateMachine::instantiate(from, handle.clone())
            .map(|sm| Box::new(sm) as Box<dyn Scene>)
            .or_else(|| {
                LinearAnimation::instantiate(from, handle).map(|la| Box::new(la) as Box<dyn Scene>)
            })
    }
}
