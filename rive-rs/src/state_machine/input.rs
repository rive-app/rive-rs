use alloc::string::String;
use core::{fmt, marker::PhantomData, ptr};

use crate::{
    ffi,
    raw_iter::{impl_iter, Raw},
};

#[derive(Clone, Copy)]
pub struct Bool<'s> {
    raw_bool: *mut ffi::Bool,
    _phantom: PhantomData<&'s ()>,
}

impl Bool<'_> {
    pub(crate) fn new(raw_bool: *mut ffi::Bool) -> Self {
        Self {
            raw_bool,
            _phantom: PhantomData,
        }
    }

    pub fn name(&self) -> String {
        let mut name = String::new();

        unsafe {
            ffi::rive_rs_input_name(self.raw_bool as *mut ffi::Input, &mut name as *mut String);
        }

        name
    }

    pub fn get(&self) -> bool {
        unsafe { ffi::rive_rs_bool_get(self.raw_bool) }
    }

    pub fn set(&mut self, val: bool) {
        unsafe {
            ffi::rive_rs_bool_set(self.raw_bool, val);
        }
    }
}

impl<'s> fmt::Debug for Bool<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Bool").field("name", &self.name()).finish()
    }
}

#[derive(Clone, Copy)]
pub struct Number<'s> {
    raw_number: *mut ffi::Number,
    _phantom: PhantomData<&'s ()>,
}

impl Number<'_> {
    pub(crate) fn new(raw_number: *mut ffi::Number) -> Self {
        Self {
            raw_number,
            _phantom: PhantomData,
        }
    }

    pub fn name(&self) -> String {
        let mut name = String::new();

        unsafe {
            ffi::rive_rs_input_name(self.raw_number as *mut ffi::Input, &mut name as *mut String);
        }

        name
    }

    pub fn get(&self) -> f32 {
        unsafe { ffi::rive_rs_number_get(self.raw_number) }
    }

    pub fn set(&mut self, val: f32) {
        unsafe {
            ffi::rive_rs_number_set(self.raw_number, val);
        }
    }
}

impl<'s> fmt::Debug for Number<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Number")
            .field("name", &self.name())
            .finish()
    }
}

#[derive(Clone, Copy)]
pub struct Trigger<'s> {
    raw_trigger: *mut ffi::Trigger,
    _phantom: PhantomData<&'s ()>,
}

impl Trigger<'_> {
    pub(crate) fn new(raw_trigger: *mut ffi::Trigger) -> Self {
        Self {
            raw_trigger,
            _phantom: PhantomData,
        }
    }

    pub fn name(&self) -> String {
        let mut name = String::new();

        unsafe {
            ffi::rive_rs_input_name(
                self.raw_trigger as *mut ffi::Input,
                &mut name as *mut String,
            );
        }

        name
    }

    pub fn fire(&mut self) {
        unsafe {
            ffi::rive_rs_trigger_fire(self.raw_trigger);
        }
    }
}

impl<'s> fmt::Debug for Trigger<'s> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Trigger")
            .field("name", &self.name())
            .finish()
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Input<'s> {
    Bool(Bool<'s>),
    Number(Number<'s>),
    Trigger(Trigger<'s>),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawStateMachine(pub *mut ffi::StateMachine);

impl Raw for RawStateMachine {
    type Item<'s> = Input<'s>;

    fn len(self) -> usize {
        unsafe { ffi::rive_rs_state_machine_input_count(self.0) }
    }

    unsafe fn get<'s>(self, index: usize) -> Self::Item<'s> {
        let mut input_tag = ffi::InputTag::Bool;
        let mut input = ptr::null_mut();
        ffi::rive_rs_state_machine_get_input(
            self.0,
            index,
            &mut input_tag as *mut ffi::InputTag,
            &mut input as *mut *mut ffi::Input,
        );

        match input_tag {
            ffi::InputTag::Bool => Input::Bool(Bool {
                raw_bool: input as *mut ffi::Bool,
                _phantom: PhantomData,
            }),
            ffi::InputTag::Number => Input::Number(Number {
                raw_number: input as *mut ffi::Number,
                _phantom: PhantomData,
            }),
            ffi::InputTag::Trigger => Input::Trigger(Trigger {
                raw_trigger: input as *mut ffi::Trigger,
                _phantom: PhantomData,
            }),
        }
    }
}

impl_iter!(InputIter, Input, RawStateMachine, 's);
