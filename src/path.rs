use core::{fmt, slice};

use crate::ffi;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FillRule {
    NonZero,
    EvenOdd,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Verb {
    Move = 0,
    Line = 1,
    Cubic = 4,
    Close = 5,
}

#[derive(Clone, Copy)]
pub struct Commands {
    raw_commands: *mut ffi::Commands,
    len: usize,
}

impl Commands {
    pub(crate) fn new(raw_commands: *mut ffi::Commands, len: usize) -> Self {
        Self { raw_commands, len }
    }
}

impl fmt::Debug for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Commands").field("len", &self.len).finish()
    }
}

impl<'c> Iterator for &'c mut Commands {
    type Item = (Verb, &'c [Point]);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.len.checked_sub(1).map(|new_len| {
            self.len = new_len;

            let ffi::Command { verb, points } =
                unsafe { ffi::rive_rs_commands_next(self.raw_commands) };

            match verb {
                Verb::Move => (verb, unsafe { slice::from_raw_parts(points, 1) }),
                Verb::Line => (verb, unsafe { slice::from_raw_parts(points.add(1), 1) }),
                Verb::Cubic => (verb, unsafe { slice::from_raw_parts(points.add(1), 3) }),
                Verb::Close => (verb, [].as_slice()),
            }
        })
    }
}
