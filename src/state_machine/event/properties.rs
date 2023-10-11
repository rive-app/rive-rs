use alloc::string::String;

#[derive(Clone, Debug)]
pub enum Property {
    Bool(bool),
    Number(f32),
    String(String),
}
