use core::{marker::PhantomData, ptr, slice, str};

use crate::{
    ffi,
    raw_iter::{impl_iter, Raw},
};

mod text_value_run;

pub use text_value_run::TextValueRun;

pub struct Component<'a> {
    raw_component: *mut ffi::Component,
    _phantom: PhantomData<&'a ()>,
}

impl Component<'_> {
    pub fn name(&self) -> &str {
        let mut data = ptr::null();
        let mut len = 0;

        let bytes = unsafe {
            ffi::rive_rs_component_name(
                self.raw_component,
                &mut data as *mut *const u8,
                &mut len as *mut usize,
            );
            slice::from_raw_parts(data, len)
        };

        str::from_utf8(bytes).expect("component name is invalid UTF-8")
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawArtboard(pub *mut ffi::Artboard);

impl Raw for RawArtboard {
    type Item<'a> = Component<'a>;

    fn len(self) -> usize {
        unsafe { ffi::rive_rs_artboard_component_count(self.0) }
    }

    unsafe fn get<'a>(self, index: usize) -> Self::Item<'a> {
        Component {
            raw_component: ffi::rive_rs_artboard_get_component(self.0, index),
            _phantom: PhantomData,
        }
    }
}

impl_iter!(Components, Component, RawArtboard, 'a);

macro_rules! try_from_component {
    ( $component:ident, $raw_name:ident, $type_id:expr ) => {
        impl<'a> TryFrom<crate::artboard::components::Component<'a>> for $component<'a> {
            type Error = ();

            fn try_from(
                value: crate::artboard::components::Component<'a>,
            ) -> Result<Self, Self::Error> {
                unsafe {
                    (crate::ffi::rive_rs_component_type_id(value.raw_component) == $type_id)
                        .then(|| Self {
                            $raw_name: value.raw_component as *mut ffi::$component,
                            _phantom: core::marker::PhantomData,
                        })
                        .ok_or(())
                }
            }
        }
    };
}

pub(crate) use try_from_component;
