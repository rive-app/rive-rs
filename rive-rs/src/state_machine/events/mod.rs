use alloc::{collections::BTreeMap, string::String};
use core::{ptr, time::Duration};

use crate::{
    ffi,
    raw_iter::{impl_iter, Raw},
};

mod properties;

pub use properties::Property;

#[derive(Clone, Debug)]
pub struct Event {
    pub name: String,
    pub delay: Duration,
    pub properties: BTreeMap<String, Property>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RawStateMachine(pub *mut ffi::StateMachine);

impl Raw for RawStateMachine {
    type Item<'s> = Event;

    fn len(self) -> usize {
        unsafe { ffi::rive_rs_state_machine_event_count(self.0) }
    }

    unsafe fn get<'s>(self, index: usize) -> Self::Item<'s> {
        let mut raw_event = ptr::null_mut();
        let mut name = String::new();
        let mut delay = 0.0;
        let mut properties = BTreeMap::new();

        unsafe {
            ffi::rive_rs_state_machine_get_event(
                self.0,
                index,
                &mut raw_event as *mut *mut ffi::Event,
                &mut delay as *mut f32,
            );
            ffi::rive_rs_event_name(raw_event, &mut name as *mut String);
            ffi::rive_rs_event_properties(
                raw_event,
                &mut properties as *mut BTreeMap<String, Property>,
            );
        }

        Event {
            name,
            delay: Duration::from_secs_f32(delay),
            properties,
        }
    }
}

impl_iter!(EventIter, Event, RawStateMachine);
