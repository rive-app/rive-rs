use alloc::{boxed::Box, collections::BTreeMap, string::String};
use core::{
    ptr::{self, NonNull},
    slice,
};

use crate::{
    linear_animation::Loop,
    path::{self, FillRule, Point, Verb},
    renderer::{
        BlendMode, Buffer, BufferFlags, BufferType, Color, Gradient, Image, Paint, PaintStyle,
        Path, Renderer, StrokeCap, StrokeJoin,
    },
    state_machine,
};

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct RawString {
    data: *const u8,
    len: usize,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Command {
    pub verb: Verb,
    pub points: *const Point,
}

#[derive(Clone, Copy)]
pub enum Commands {}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub enum FileResult {
    Success,
    #[allow(dead_code)]
    UnsupportedVersion,
    #[allow(dead_code)]
    Malformed,
}

#[derive(Clone, Copy)]
pub enum Factory {}

#[derive(Clone, Copy)]
pub enum File {}

#[derive(Clone, Copy)]
pub enum Artboard {}

#[derive(Clone, Copy)]
pub enum Component {}

#[derive(Clone, Copy)]
pub enum TextValueRun {}

#[derive(Clone, Copy)]
pub enum LinearAnimation {}

#[derive(Clone, Copy)]
pub enum StateMachine {}

#[derive(Clone, Copy)]
pub enum Scene {}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum PropertyTag {
    Bool,
    Number,
    String,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union Property {
    bool: bool,
    number: f32,
    string: RawString,
}

#[derive(Clone, Copy)]
pub enum Event {}

#[derive(Clone, Copy)]
pub enum Input {}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum InputTag {
    Bool,
    Number,
    Trigger,
}

#[derive(Clone, Copy)]
pub enum Bool {}

#[derive(Clone, Copy)]
pub enum Number {}

#[derive(Clone, Copy)]
pub enum Trigger {}

#[no_mangle]
unsafe extern "C" fn rive_rs_allocate_string(string: *mut String, data: *const u8, len: usize) {
    let string = &mut *string;

    if let Ok(converted) = String::from_utf8(slice::from_raw_parts(data, len).to_vec()) {
        *string = converted;
    }
}

#[no_mangle]
unsafe extern "C" fn rive_rs_insert_property(
    properties: *mut BTreeMap<String, state_machine::Property>,
    key_data: *const u8,
    key_len: usize,
    value_tag: PropertyTag,
    value_payload: Property,
) {
    let properties = &mut *properties;

    if let Ok(key) = String::from_utf8(slice::from_raw_parts(key_data, key_len).to_vec()) {
        let value = match value_tag {
            PropertyTag::Bool => state_machine::Property::Bool(value_payload.bool),
            PropertyTag::Number => state_machine::Property::Number(value_payload.number),
            PropertyTag::String => {
                if let Ok(string) = String::from_utf8(
                    slice::from_raw_parts(value_payload.string.data, value_payload.string.len)
                        .to_vec(),
                ) {
                    state_machine::Property::String(string)
                } else {
                    return;
                }
            }
        };

        properties.insert(key, value);
    }
}

#[repr(C)]
pub struct RendererEntries<R: Renderer> {
    buffer_new: unsafe extern "C" fn(BufferType, BufferFlags, usize) -> *mut R::Buffer,
    buffer_release: unsafe extern "C" fn(*mut R::Buffer),
    buffer_map: unsafe extern "C" fn(*mut R::Buffer) -> *mut u8,
    buffer_unmap: unsafe extern "C" fn(*mut R::Buffer),
    path_default: unsafe extern "C" fn() -> *mut R::Path,
    path_new: unsafe extern "C" fn(*mut Commands, usize, FillRule) -> *mut R::Path,
    path_release: unsafe extern "C" fn(*mut R::Path),
    path_reset: unsafe extern "C" fn(*mut R::Path),
    path_extend: unsafe extern "C" fn(*mut R::Path, *const R::Path, transform: *const [f32; 6]),
    path_set_fill_rule: unsafe extern "C" fn(*mut R::Path, FillRule),
    path_move_to: unsafe extern "C" fn(*mut R::Path, f32, f32),
    path_line_to: unsafe extern "C" fn(*mut R::Path, f32, f32),
    path_cubic_to: unsafe extern "C" fn(*mut R::Path, f32, f32, f32, f32, f32, f32),
    path_close: unsafe extern "C" fn(*mut R::Path),
    paint_default: unsafe extern "C" fn() -> *mut R::Paint,
    paint_release: unsafe extern "C" fn(*mut R::Paint),
    paint_set_style: unsafe extern "C" fn(*mut R::Paint, PaintStyle),
    paint_set_color: unsafe extern "C" fn(*mut R::Paint, Color),
    paint_set_thickness: unsafe extern "C" fn(*mut R::Paint, f32),
    paint_set_join: unsafe extern "C" fn(*mut R::Paint, StrokeJoin),
    paint_set_cap: unsafe extern "C" fn(*mut R::Paint, StrokeCap),
    paint_set_blend_mode: unsafe extern "C" fn(*mut R::Paint, BlendMode),
    paint_set_gradient: unsafe extern "C" fn(*mut R::Paint, *const R::Gradient),
    paint_invalidate_stroke: unsafe extern "C" fn(*mut R::Paint),
    gradient_new_linear: unsafe extern "C" fn(
        f32,
        f32,
        f32,
        f32,
        *const Color,
        *const f32,
        usize,
    ) -> *mut R::Gradient,
    gradient_new_radial:
        unsafe extern "C" fn(f32, f32, f32, *const Color, *const f32, usize) -> *mut R::Gradient,
    gradient_release: unsafe extern "C" fn(*mut R::Gradient),
    image_decode: unsafe extern "C" fn(*const u8, usize) -> *mut R::Image,
    image_release: unsafe extern "C" fn(*mut R::Image),
    renderer_state_push: unsafe extern "C" fn(*mut R),
    renderer_state_pop: unsafe extern "C" fn(*mut R),
    renderer_transform: unsafe extern "C" fn(*mut R, transform: *const [f32; 6]),
    renderer_set_clip: unsafe extern "C" fn(*mut R, path: *const R::Path),
    renderer_draw_path: unsafe extern "C" fn(*mut R, path: *const R::Path, paint: *const R::Paint),
    renderer_draw_image:
        unsafe extern "C" fn(*mut R, image: *const R::Image, blend_mode: BlendMode, opacity: f32),
    renderer_draw_image_mesh: unsafe extern "C" fn(
        *mut R,
        image: *const R::Image,
        vertices: *const R::Buffer,
        uvs: *const R::Buffer,
        indices: *const R::Buffer,
        blend_mode: BlendMode,
        opacity: f32,
    ),
}

impl<R: Renderer> RendererEntries<R> {
    pub const ENTRIES: &'static Self = {
        unsafe extern "C" fn buffer_new<R: Renderer>(
            r#type: BufferType,
            flags: BufferFlags,
            len_in_bytes: usize,
        ) -> *mut R::Buffer {
            Box::into_raw(Box::new(R::Buffer::new(r#type, flags, len_in_bytes)))
        }

        unsafe extern "C" fn buffer_release<R: Renderer>(buffer: *mut R::Buffer) {
            drop(Box::from_raw(buffer))
        }

        unsafe extern "C" fn buffer_map<R: Renderer>(buffer: *mut R::Buffer) -> *mut u8 {
            (*buffer).map().as_mut_ptr()
        }

        unsafe extern "C" fn buffer_unmap<R: Renderer>(buffer: *mut R::Buffer) {
            (*buffer).unmap();
        }

        unsafe extern "C" fn path_default<R: Renderer>() -> *mut R::Path {
            Box::into_raw(Box::default())
        }

        unsafe extern "C" fn path_new<R: Renderer>(
            commands: *mut Commands,
            len: usize,
            fill_rule: FillRule,
        ) -> *mut R::Path {
            Box::into_raw(Box::new(R::Path::new(
                &mut path::Commands::new(commands, len),
                fill_rule,
            )))
        }

        unsafe extern "C" fn path_release<R: Renderer>(path: *mut R::Path) {
            drop(Box::from_raw(path))
        }

        unsafe extern "C" fn path_reset<R: Renderer>(path: *mut R::Path) {
            (*path).reset();
        }

        unsafe extern "C" fn path_extend<R: Renderer>(
            path: *mut R::Path,
            other: *const R::Path,
            transform: *const [f32; 6],
        ) {
            (*path).extend(&*other, &*transform);
        }

        unsafe extern "C" fn path_set_fill_rule<R: Renderer>(
            path: *mut R::Path,
            fill_rule: FillRule,
        ) {
            (*path).set_fill_rule(fill_rule);
        }

        unsafe extern "C" fn path_move_to<R: Renderer>(path: *mut R::Path, x: f32, y: f32) {
            (*path).move_to(x, y);
        }

        unsafe extern "C" fn path_line_to<R: Renderer>(path: *mut R::Path, x: f32, y: f32) {
            (*path).line_to(x, y);
        }

        unsafe extern "C" fn path_cubic_to<R: Renderer>(
            path: *mut R::Path,
            ox: f32,
            oy: f32,
            ix: f32,
            iy: f32,
            x: f32,
            y: f32,
        ) {
            (*path).cubic_to(ox, oy, ix, iy, x, y);
        }

        unsafe extern "C" fn path_close<R: Renderer>(path: *mut R::Path) {
            (*path).close();
        }

        unsafe extern "C" fn paint_default<R: Renderer>() -> *mut R::Paint {
            Box::into_raw(Box::default())
        }

        unsafe extern "C" fn paint_release<R: Renderer>(paint: *mut R::Paint) {
            drop(Box::from_raw(paint))
        }

        unsafe extern "C" fn paint_set_style<R: Renderer>(paint: *mut R::Paint, style: PaintStyle) {
            (*paint).set_style(style);
        }

        unsafe extern "C" fn paint_set_color<R: Renderer>(paint: *mut R::Paint, color: Color) {
            (*paint).set_color(color);
        }

        unsafe extern "C" fn paint_set_thickness<R: Renderer>(
            paint: *mut R::Paint,
            thickness: f32,
        ) {
            (*paint).set_thickness(thickness);
        }

        unsafe extern "C" fn paint_set_join<R: Renderer>(paint: *mut R::Paint, join: StrokeJoin) {
            (*paint).set_join(join);
        }

        unsafe extern "C" fn paint_set_cap<R: Renderer>(paint: *mut R::Paint, cap: StrokeCap) {
            (*paint).set_cap(cap);
        }

        unsafe extern "C" fn paint_set_blend_mode<R: Renderer>(
            paint: *mut R::Paint,
            blend_mode: BlendMode,
        ) {
            (*paint).set_blend_mode(blend_mode);
        }

        unsafe extern "C" fn paint_set_gradient<R: Renderer>(
            paint: *mut R::Paint,
            gradient: *const R::Gradient,
        ) {
            (*paint).set_gradient(&*gradient);
        }

        unsafe extern "C" fn paint_invalidate_stroke<R: Renderer>(paint: *mut R::Paint) {
            (*paint).invalidate_stroke();
        }

        unsafe extern "C" fn gradient_new_linear<R: Renderer>(
            sx: f32,
            sy: f32,
            ex: f32,
            ey: f32,
            colors: *const Color,
            stops: *const f32,
            len: usize,
        ) -> *mut R::Gradient {
            Box::into_raw(Box::new(R::Gradient::new_linear(
                sx,
                sy,
                ex,
                ey,
                slice::from_raw_parts(colors, len),
                slice::from_raw_parts(stops, len),
            )))
        }

        unsafe extern "C" fn gradient_new_radial<R: Renderer>(
            cx: f32,
            cy: f32,
            radius: f32,
            colors: *const Color,
            stops: *const f32,
            len: usize,
        ) -> *mut R::Gradient {
            Box::into_raw(Box::new(R::Gradient::new_radial(
                cx,
                cy,
                radius,
                slice::from_raw_parts(colors, len),
                slice::from_raw_parts(stops, len),
            )))
        }

        unsafe extern "C" fn gradient_release<R: Renderer>(gradient: *mut R::Gradient) {
            drop(Box::from_raw(gradient))
        }

        unsafe extern "C" fn image_deocde<R: Renderer>(
            data: *const u8,
            len: usize,
        ) -> *mut R::Image {
            R::Image::decode(slice::from_raw_parts(data, len))
                .map(|image| Box::into_raw(Box::new(image)))
                .unwrap_or(ptr::null_mut())
        }

        unsafe extern "C" fn image_release<R: Renderer>(image: *mut R::Image) {
            drop(Box::from_raw(image))
        }

        unsafe extern "C" fn renderer_state_push<R: Renderer>(renderer: *mut R) {
            (*renderer).state_push();
        }

        unsafe extern "C" fn renderer_state_pop<R: Renderer>(renderer: *mut R) {
            (*renderer).state_pop();
        }

        unsafe extern "C" fn renderer_transform<R: Renderer>(
            renderer: *mut R,
            transform: *const [f32; 6],
        ) {
            (*renderer).transform(&*transform);
        }

        unsafe extern "C" fn renderer_set_clip<R: Renderer>(
            renderer: *mut R,
            path: *const R::Path,
        ) {
            (*renderer).set_clip(&*path);
        }

        unsafe extern "C" fn renderer_draw_path<R: Renderer>(
            renderer: *mut R,
            path: *const R::Path,
            paint: *const R::Paint,
        ) {
            (*renderer).draw_path(&*path, &*paint);
        }

        unsafe extern "C" fn renderer_draw_image<R: Renderer>(
            renderer: *mut R,
            image: *const R::Image,
            blend_mode: BlendMode,
            opacity: f32,
        ) {
            (*renderer).draw_image(&*image, blend_mode, opacity);
        }

        unsafe extern "C" fn renderer_draw_image_mesh<R: Renderer>(
            renderer: *mut R,
            image: *const R::Image,
            vertices: *const R::Buffer,
            uvs: *const R::Buffer,
            indices: *const R::Buffer,
            blend_mode: BlendMode,
            opacity: f32,
        ) {
            (*renderer).draw_image_mesh(&*image, &*vertices, &*uvs, &*indices, blend_mode, opacity);
        }

        &Self {
            buffer_new: buffer_new::<R>,
            buffer_release: buffer_release::<R>,
            buffer_map: buffer_map::<R>,
            buffer_unmap: buffer_unmap::<R>,
            path_default: path_default::<R>,
            path_new: path_new::<R>,
            path_release: path_release::<R>,
            path_reset: path_reset::<R>,
            path_extend: path_extend::<R>,
            path_set_fill_rule: path_set_fill_rule::<R>,
            path_move_to: path_move_to::<R>,
            path_line_to: path_line_to::<R>,
            path_cubic_to: path_cubic_to::<R>,
            path_close: path_close::<R>,
            paint_default: paint_default::<R>,
            paint_release: paint_release::<R>,
            paint_set_style: paint_set_style::<R>,
            paint_set_color: paint_set_color::<R>,
            paint_set_thickness: paint_set_thickness::<R>,
            paint_set_join: paint_set_join::<R>,
            paint_set_cap: paint_set_cap::<R>,
            paint_set_blend_mode: paint_set_blend_mode::<R>,
            paint_set_gradient: paint_set_gradient::<R>,
            paint_invalidate_stroke: paint_invalidate_stroke::<R>,
            gradient_new_linear: gradient_new_linear::<R>,
            gradient_new_radial: gradient_new_radial::<R>,
            gradient_release: gradient_release::<R>,
            image_decode: image_deocde::<R>,
            image_release: image_release::<R>,
            renderer_state_push: renderer_state_push::<R>,
            renderer_state_pop: renderer_state_pop::<R>,
            renderer_transform: renderer_transform::<R>,
            renderer_set_clip: renderer_set_clip::<R>,
            renderer_draw_path: renderer_draw_path::<R>,
            renderer_draw_image: renderer_draw_image::<R>,
            renderer_draw_image_mesh: renderer_draw_image_mesh::<R>,
        }
    };
}

extern "C" {
    #[allow(improper_ctypes)]
    pub fn rive_rs_file_new(
        data: *const u8,
        len: usize,
        entries: *const (),
        result: *mut FileResult,
        factory: *mut *mut Factory,
    ) -> *const File;
    pub fn rive_rs_file_release(file: *const File, factory: *mut Factory);
    pub fn rive_rs_instantiate_artboard(
        file: *const File,
        index: Option<NonNull<usize>>,
        artboard: *mut Option<NonNull<Artboard>>,
    );
    pub fn rive_rs_instantiate_artboard_by_name(
        file: *const File,
        data: *const u8,
        len: usize,
        raw_artboard: *mut Option<NonNull<Artboard>>,
    );
    pub fn rive_rs_artboard_instance_release(artboard_instance: *mut Artboard);
    pub fn rive_rs_artboard_component_count(artboard_instance: *mut Artboard) -> usize;
    pub fn rive_rs_artboard_get_component(
        artboard_instance: *mut Artboard,
        index: usize,
    ) -> *mut Component;
    pub fn rive_rs_component_type_id(component: *const Component) -> u16;
    pub fn rive_rs_component_name(
        component: *const Component,
        data: *mut *const u8,
        len: *mut usize,
    );
    pub fn rive_rs_text_value_run_get_text(
        text_value_run: *const TextValueRun,
        data: *mut *const u8,
        len: *mut usize,
    );
    pub fn rive_rs_text_value_run_set_text(
        text_value_run: *mut TextValueRun,
        data: *const u8,
        len: usize,
    );
    pub fn rive_rs_instantiate_linear_animation(
        artboard: *mut Artboard,
        index: Option<NonNull<usize>>,
        linear_animation: *mut Option<NonNull<LinearAnimation>>,
    );
    pub fn rive_rs_instantiate_linear_animation_by_name(
        artboard: *mut Artboard,
        data: *const u8,
        len: usize,
        linear_animation: *mut Option<NonNull<LinearAnimation>>,
    );
    pub fn rive_rs_linear_animation_time(linear_animation: *mut LinearAnimation) -> f32;
    pub fn rive_rs_linear_animation_set_time(linear_animation: *mut LinearAnimation, time: f32);
    pub fn rive_rs_linear_animation_is_forwards(linear_animation: *mut LinearAnimation) -> bool;
    pub fn rive_rs_linear_animation_set_is_forwards(
        linear_animation: *mut LinearAnimation,
        is_forwards: bool,
    );
    pub fn rive_rs_linear_animation_advance(
        linear_animation: *mut LinearAnimation,
        elapsed: f32,
    ) -> bool;
    pub fn rive_rs_linear_animation_apply(linear_animation: *mut LinearAnimation, mix: f32);
    pub fn rive_rs_linear_animation_did_loop(linear_animation: *mut LinearAnimation) -> bool;
    pub fn rive_rs_linear_animation_set_loop(linear_animation: *mut LinearAnimation, r#loop: Loop);
    pub fn rive_rs_linear_animation_is_done(linear_animation: *mut LinearAnimation) -> bool;
    pub fn rive_rs_instantiate_state_machine(
        artboard: *mut Artboard,
        index: Option<NonNull<usize>>,
        state_machine: *mut Option<NonNull<StateMachine>>,
    );
    pub fn rive_rs_instantiate_state_machine_by_name(
        artboard: *mut Artboard,
        data: *const u8,
        len: usize,
        state_machine: *mut Option<NonNull<StateMachine>>,
    );
    pub fn rive_rs_state_machine_get_event(
        state_machine: *mut StateMachine,
        index: usize,
        input: *mut *mut Event,
        delay: *mut f32,
    );
    pub fn rive_rs_state_machine_event_count(state_machine: *mut StateMachine) -> usize;
    #[allow(improper_ctypes)]
    pub fn rive_rs_event_name(event: *mut Event, string: *mut String);
    #[allow(improper_ctypes)]
    pub fn rive_rs_event_properties(
        event: *mut Event,
        properties: *mut BTreeMap<String, state_machine::Property>,
    );
    pub fn rive_rs_state_machine_get_input(
        state_machine: *mut StateMachine,
        index: usize,
        input_tag: *mut InputTag,
        input: *mut *mut Input,
    );
    pub fn rive_rs_state_machine_input_count(state_machine: *mut StateMachine) -> usize;
    pub fn rive_rs_state_machine_get_bool(
        state_machine: *mut StateMachine,
        name: *const u8,
        len: usize,
    ) -> *mut Bool;
    pub fn rive_rs_state_machine_get_number(
        state_machine: *mut StateMachine,
        name: *const u8,
        len: usize,
    ) -> *mut Number;
    pub fn rive_rs_state_machine_get_trigger(
        state_machine: *mut StateMachine,
        name: *const u8,
        len: usize,
    ) -> *mut Trigger;
    pub fn rive_rs_input_name(input: *mut Input, data: *mut *const u8, len: *mut usize);
    pub fn rive_rs_bool_get(bool: *mut Bool) -> bool;
    pub fn rive_rs_bool_set(bool: *mut Bool, val: bool);
    pub fn rive_rs_number_get(number: *mut Number) -> f32;
    pub fn rive_rs_number_set(number: *mut Number, val: f32);
    pub fn rive_rs_trigger_fire(trigger: *mut Trigger);
    pub fn rive_rs_artboard_instance_transforms(
        artboard_instance: *mut Artboard,
        width: u32,
        height: u32,
        view_transform: *mut f32,
        inverse_view_transform: *mut f32,
    );
    pub fn rive_rs_scene_release(scene: *mut Scene);
    pub fn rive_rs_commands_next(commands: *mut Commands) -> Command;
    pub fn rive_rs_scene_width(scene: *mut Scene) -> f32;
    pub fn rive_rs_scene_height(scene: *mut Scene) -> f32;
    pub fn rive_rs_scene_loop(scene: *mut Scene) -> Loop;
    pub fn rive_rs_scene_is_translucent(scene: *mut Scene) -> bool;
    pub fn rive_rs_scene_duration(scene: *mut Scene) -> f32;
    pub fn rive_rs_scene_advance_and_apply(scene: *mut Scene, elapsed: f32) -> bool;
    pub fn rive_rs_scene_draw(scene: *mut Scene, renderer: *mut (), entries: *const ());
    pub fn rive_rs_scene_pointer_down(scene: *mut Scene, x: f32, y: f32);
    pub fn rive_rs_scene_pointer_move(scene: *mut Scene, x: f32, y: f32);
    pub fn rive_rs_scene_pointer_up(scene: *mut Scene, x: f32, y: f32);
}
