use core::{marker::PhantomData, ptr, slice, str};

use crate::ffi;

use super::try_from_component;

pub struct TextValueRun<'a> {
    raw_text_value_run: *mut ffi::TextValueRun,
    _phantom: PhantomData<&'a ()>,
}

impl TextValueRun<'_> {
    pub fn get_text(&self) -> &str {
        let mut data = ptr::null();
        let mut len = 0;

        let bytes = unsafe {
            ffi::rive_rs_text_value_run_get_text(
                self.raw_text_value_run,
                &mut data as *mut *const u8,
                &mut len as *mut usize,
            );
            slice::from_raw_parts(data, len)
        };

        str::from_utf8(bytes).expect("text value run text is invalid UTF-8")
    }

    pub fn set_text(&mut self, text: &str) {
        unsafe {
            ffi::rive_rs_text_value_run_set_text(
                self.raw_text_value_run,
                text.as_ptr(),
                text.len(),
            );
        }
    }
}

try_from_component!(TextValueRun, raw_text_value_run, 135);
