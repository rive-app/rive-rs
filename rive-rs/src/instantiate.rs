use alloc::{borrow::Cow, string::String};

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Handle {
    #[default]
    Default,
    Index(usize),
    Name(Cow<'static, String>),
}

pub trait Instantiate: Sized {
    type From;

    fn instantiate(from: &Self::From, handle: Handle) -> Option<Self>;
}
