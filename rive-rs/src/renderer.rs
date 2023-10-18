use bitflags::bitflags;

use crate::path::{Commands, FillRule};

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BufferType {
    Index,
    Vertex,
}

bitflags! {
    #[repr(C)]
    #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
    pub struct BufferFlags: u32 {
        const NONE = 0;
        const MAPPED_ONCE_AT_INITIALIZATION = 1;
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StrokeJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StrokeCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BlendMode {
    SrcOver = 3,
    Screen = 14,
    Overlay = 15,
    Darken = 16,
    Lighten = 17,
    ColorDodge = 18,
    ColorBurn = 19,
    HardLight = 20,
    SoftLight = 21,
    Difference = 22,
    Exclusion = 23,
    Multiply = 24,
    Hue = 25,
    Saturation = 26,
    Color = 27,
    Luminosity = 28,
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PaintStyle {
    Stroke,
    Fill,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Color {
    pub b: u8,
    pub g: u8,
    pub r: u8,
    pub a: u8,
}

pub trait Buffer: Sized {
    fn new(r#type: BufferType, flags: BufferFlags, len_in_bytes: usize) -> Self;
    fn map(&mut self) -> &mut [u8];
    fn unmap(&mut self);
}

pub trait Path: Default + Sized {
    fn new(commands: &mut Commands, fill_rule: FillRule) -> Self;
    fn reset(&mut self);
    fn extend(&mut self, from: &Self, transform: &[f32; 6]);
    fn set_fill_rule(&mut self, fill_rule: FillRule);
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32);
    fn close(&mut self);
}

pub trait Paint: Default + Sized {
    type Gradient: Gradient;

    fn set_style(&mut self, style: PaintStyle);
    fn set_color(&mut self, color: Color);
    fn set_thickness(&mut self, thickness: f32);
    fn set_join(&mut self, join: StrokeJoin);
    fn set_cap(&mut self, cap: StrokeCap);
    fn set_blend_mode(&mut self, blend_mode: BlendMode);
    fn set_gradient(&mut self, gradient: &Self::Gradient);
    fn invalidate_stroke(&mut self);
}

pub trait Gradient: Sized {
    fn new_linear(sx: f32, sy: f32, ex: f32, ey: f32, colors: &[Color], stops: &[f32]) -> Self;
    fn new_radial(cx: f32, cy: f32, radius: f32, colors: &[Color], stops: &[f32]) -> Self;
}

pub trait Image: Sized {
    fn decode(data: &[u8]) -> Option<Self>;
}

pub trait Renderer: Sized + 'static {
    type Buffer: Buffer;
    type Path: Path;
    type Paint: Paint<Gradient = Self::Gradient>;
    type Gradient: Gradient;
    type Image: Image;

    fn state_push(&mut self);
    fn state_pop(&mut self);
    fn transform(&mut self, transform: &[f32; 6]);
    fn set_clip(&mut self, path: &Self::Path);
    fn draw_path(&mut self, path: &Self::Path, paint: &Self::Paint);
    fn draw_image(&mut self, image: &Self::Image, blend_mode: BlendMode, opacity: f32);
    fn draw_image_mesh(
        &mut self,
        image: &Self::Image,
        vertices: &Self::Buffer,
        uvs: &Self::Buffer,
        indices: &Self::Buffer,
        blend_mode: BlendMode,
        opacity: f32,
    );
}
