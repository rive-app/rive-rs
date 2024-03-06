use std::{fmt, io::Cursor};

use image::io::Reader;
use smallvec::SmallVec;
use vello::{
    kurbo::{Affine, BezPath, Cap, Join, Line, PathSeg, Point, Rect, Shape, Stroke, Vec2},
    peniko::{
        self, BlendMode, Brush, BrushRef, Color, ColorStop, ColorStopsSource, Fill, Format, Mix,
    },
    Scene,
};

mod util;

use util::ScaleFromOrigin;

use crate::renderer;

fn to_vello_color(color: renderer::Color) -> Color {
    Color::rgba8(color.r, color.g, color.b, color.a)
}

fn to_vello_mix(blend_mode: renderer::BlendMode) -> Mix {
    match blend_mode {
        renderer::BlendMode::SrcOver => Mix::Normal,
        renderer::BlendMode::Screen => Mix::Screen,
        renderer::BlendMode::Overlay => Mix::Overlay,
        renderer::BlendMode::Darken => Mix::Darken,
        renderer::BlendMode::Lighten => Mix::Lighten,
        renderer::BlendMode::ColorDodge => Mix::ColorDodge,
        renderer::BlendMode::ColorBurn => Mix::ColorBurn,
        renderer::BlendMode::HardLight => Mix::HardLight,
        renderer::BlendMode::SoftLight => Mix::SoftLight,
        renderer::BlendMode::Difference => Mix::Difference,
        renderer::BlendMode::Exclusion => Mix::Exclusion,
        renderer::BlendMode::Multiply => Mix::Multiply,
        renderer::BlendMode::Hue => Mix::Hue,
        renderer::BlendMode::Saturation => Mix::Saturation,
        renderer::BlendMode::Color => Mix::Color,
        renderer::BlendMode::Luminosity => Mix::Luminosity,
    }
}

fn triangle_path(points: [Point; 3]) -> BezPath {
    BezPath::from_path_segments(
        [
            PathSeg::Line(Line::new(points[0], points[1])),
            PathSeg::Line(Line::new(points[1], points[2])),
            PathSeg::Line(Line::new(points[2], points[0])),
        ]
        .into_iter(),
    )
}

#[derive(Debug)]
pub struct Buffer {
    inner: Vec<u8>,
}

impl Buffer {
    #[inline]
    pub fn as_f32_pairs(&self) -> &[[f32; 2]] {
        bytemuck::cast_slice(&self.inner)
    }

    #[inline]
    pub fn as_u16s(&self) -> &[u16] {
        bytemuck::cast_slice(&self.inner)
    }
}

impl renderer::Buffer for Buffer {
    #[inline]
    fn new(
        _type: renderer::BufferType,
        _flags: renderer::BufferFlags,
        len_in_bytes: usize,
    ) -> Self {
        Self {
            inner: vec![0; len_in_bytes],
        }
    }

    #[inline]
    fn map(&mut self) -> &mut [u8] {
        &mut self.inner
    }

    #[inline]
    fn unmap(&mut self) {}
}

#[derive(Debug)]
pub struct Path {
    inner: BezPath,
    fill: Fill,
}

impl Default for Path {
    #[inline]
    fn default() -> Self {
        Self {
            inner: Default::default(),
            fill: Fill::NonZero,
        }
    }
}

impl renderer::Path for Path {
    fn new(commands: &mut crate::path::Commands, fill_rule: crate::path::FillRule) -> Self {
        let mut path = Self::default();

        for (verb, points) in commands {
            match verb {
                crate::path::Verb::Move => path.move_to(points[0].x, points[0].y),
                crate::path::Verb::Line => path.line_to(points[0].x, points[0].y),
                crate::path::Verb::Cubic => path.cubic_to(
                    points[0].x,
                    points[0].y,
                    points[1].x,
                    points[1].y,
                    points[2].x,
                    points[2].y,
                ),
                crate::path::Verb::Close => path.close(),
            }
        }

        path.set_fill_rule(fill_rule);

        path
    }

    #[inline]
    fn reset(&mut self) {
        self.inner.truncate(0);
    }

    #[inline]
    fn extend(&mut self, from: &Self, transform: &[f32; 6]) {
        let mut from = from.inner.clone();
        from.apply_affine(Affine::new(transform.map(Into::into)));

        self.inner.extend(from.elements().iter().cloned());
    }

    #[inline]
    fn set_fill_rule(&mut self, fill_rule: crate::path::FillRule) {
        self.fill = match fill_rule {
            crate::path::FillRule::NonZero => Fill::NonZero,
            crate::path::FillRule::EvenOdd => Fill::EvenOdd,
        };
    }

    #[inline]
    fn move_to(&mut self, x: f32, y: f32) {
        self.inner.move_to(Point::new(x as f64, y as f64));
    }

    #[inline]
    fn line_to(&mut self, x: f32, y: f32) {
        self.inner.line_to(Point::new(x as f64, y as f64));
    }

    #[inline]
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.inner.curve_to(
            Point::new(ox as f64, oy as f64),
            Point::new(ix as f64, iy as f64),
            Point::new(x as f64, y as f64),
        );
    }

    #[inline]
    fn close(&mut self) {
        self.inner.close_path();
    }
}

#[derive(Debug)]
struct SliceStops<'s> {
    colors: &'s [renderer::Color],
    stops: &'s [f32],
}

impl ColorStopsSource for SliceStops<'_> {
    fn collect_stops(&self, vec: &mut SmallVec<[ColorStop; 4]>) {
        vec.extend(
            self.colors
                .iter()
                .zip(self.stops.iter())
                .map(|(&color, &offset)| ColorStop {
                    offset,
                    color: to_vello_color(color),
                }),
        );
    }
}

#[derive(Debug)]
enum RenderStyle {
    Fill,
    Stroke(Stroke),
}

#[derive(Debug)]
pub struct Paint {
    style: RenderStyle,
    brush: Brush,
    blend_mode: BlendMode,
}

impl Default for Paint {
    #[inline]
    fn default() -> Self {
        Self {
            style: RenderStyle::Fill,
            brush: Brush::Solid(Color::TRANSPARENT),
            blend_mode: Mix::Normal.into(),
        }
    }
}

impl renderer::Paint for Paint {
    type Gradient = Gradient;

    #[inline]
    fn set_style(&mut self, style: renderer::PaintStyle) {
        self.style = match style {
            renderer::PaintStyle::Stroke => RenderStyle::Stroke(Stroke::new(0.0)),
            renderer::PaintStyle::Fill => RenderStyle::Fill,
        }
    }

    #[inline]
    fn set_color(&mut self, color: renderer::Color) {
        self.brush = Brush::Solid(to_vello_color(color));
    }

    #[inline]
    fn set_thickness(&mut self, thickness: f32) {
        loop {
            if let RenderStyle::Stroke(stroke) = &mut self.style {
                stroke.width = thickness as f64;
                break;
            } else {
                self.style = RenderStyle::Stroke(Stroke::new(0.0));
            }
        }
    }

    #[inline]
    fn set_join(&mut self, join: renderer::StrokeJoin) {
        loop {
            if let RenderStyle::Stroke(stroke) = &mut self.style {
                stroke.join = match join {
                    renderer::StrokeJoin::Miter => Join::Miter,
                    renderer::StrokeJoin::Round => Join::Round,
                    renderer::StrokeJoin::Bevel => Join::Bevel,
                };
                break;
            } else {
                self.style = RenderStyle::Stroke(Stroke::new(0.0));
            }
        }
    }

    #[inline]
    fn set_cap(&mut self, cap: renderer::StrokeCap) {
        loop {
            if let RenderStyle::Stroke(stroke) = &mut self.style {
                stroke.start_cap = match cap {
                    renderer::StrokeCap::Butt => Cap::Butt,
                    renderer::StrokeCap::Round => Cap::Round,
                    renderer::StrokeCap::Square => Cap::Square,
                };
                stroke.end_cap = stroke.start_cap;
                break;
            } else {
                self.style = RenderStyle::Stroke(Stroke::new(0.0));
            }
        }
    }

    #[inline]
    fn set_blend_mode(&mut self, blend_mode: renderer::BlendMode) {
        self.blend_mode = to_vello_mix(blend_mode).into();
    }

    #[inline]
    fn set_gradient(&mut self, gradient: &Self::Gradient) {
        self.brush = Brush::Gradient(gradient.inner.clone());
    }

    #[inline]
    fn invalidate_stroke(&mut self) {}
}

#[derive(Debug)]
pub struct Gradient {
    inner: peniko::Gradient,
}

impl renderer::Gradient for Gradient {
    #[inline]
    fn new_linear(
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[renderer::Color],
        stops: &[f32],
    ) -> Self {
        let stops = SliceStops { colors, stops };
        Gradient {
            inner: peniko::Gradient::new_linear((sx as f64, sy as f64), (ex as f64, ey as f64))
                .with_stops(stops),
        }
    }

    #[inline]
    fn new_radial(
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[renderer::Color],
        stops: &[f32],
    ) -> Self {
        let stops = SliceStops { colors, stops };
        Gradient {
            inner: peniko::Gradient::new_radial((cx as f64, cy as f64), radius).with_stops(stops),
        }
    }
}

#[derive(Debug)]
pub struct Image {
    inner: peniko::Image,
}

impl renderer::Image for Image {
    fn decode(data: &[u8]) -> Option<Self> {
        let image = Reader::new(Cursor::new(data))
            .with_guessed_format()
            .ok()?
            .decode()
            .ok()?
            .into_rgba8();
        let width = image.width();
        let height = image.height();

        Some(Image {
            inner: peniko::Image::new(image.into_raw().into(), Format::Rgba8, width, height),
        })
    }
}

pub struct Renderer {
    scene: Box<Scene>,
    // scene: SceneBuilder<'static>,
    transforms: Vec<Affine>,
    clips: Vec<bool>,
}

impl Renderer {
    pub fn scene(&self) -> &Scene {
        &self.scene
    }

    pub fn into_scene(self) -> Scene {
        *self.scene
    }

    fn last_transform(&mut self) -> &mut Affine {
        self.transforms.last_mut().unwrap()
    }

    fn last_clip(&mut self) -> &mut bool {
        self.clips.last_mut().unwrap()
    }
}

impl Default for Renderer {
    #[inline]
    fn default() -> Self {
        let mut scene = Box::<Scene>::default();

        Self {
            scene,
            transforms: vec![Affine::IDENTITY],
            clips: vec![false],
        }
    }
}

impl renderer::Renderer for Renderer {
    type Buffer = Buffer;

    type Path = Path;

    type Paint = Paint;

    type Gradient = Gradient;

    type Image = Image;

    #[inline]
    fn state_push(&mut self) {
        let last_transform = *self.last_transform();

        self.transforms.push(last_transform);
        self.clips.push(false);
    }

    #[inline]
    fn state_pop(&mut self) {
        self.transforms.pop();
        if self.clips.pop().unwrap_or_default() {
            self.scene.pop_layer();
        }

        if self.transforms.is_empty() {
            self.transforms.push(Affine::IDENTITY);
            self.clips.push(false);
        }
    }

    #[inline]
    fn transform(&mut self, transform: &[f32; 6]) {
        let last_transform = self.last_transform();
        *last_transform *= Affine::new((*transform).map(Into::into));
    }

    #[inline]
    fn set_clip(&mut self, path: &Self::Path) {
        let transform = *self.last_transform();

        if *self.last_clip() {
            self.scene.pop_layer();
        }

        self.scene
            .push_layer(Mix::Clip, 1.0, transform, &path.inner);

        *self.last_clip() = true;
    }

    #[inline]
    fn draw_path(&mut self, path: &Self::Path, paint: &Self::Paint) {
        let transform = *self.last_transform();

        let builder = &mut self.scene;

        let skip_blending = paint.blend_mode == Mix::Normal.into();

        if !skip_blending {
            builder.push_layer(paint.blend_mode, 1.0, transform, &path.inner.bounding_box());
        }

        match &paint.style {
            RenderStyle::Fill => {
                builder.fill(path.fill, transform, &paint.brush, None, &path.inner)
            }
            RenderStyle::Stroke(stroke) => {
                builder.stroke(stroke, transform, &paint.brush, None, &path.inner)
            }
        }

        if !skip_blending {
            builder.pop_layer();
        }
    }

    #[inline]
    fn draw_image(&mut self, image: &Self::Image, blend_mode: renderer::BlendMode, opacity: f32) {
        let image = &image.inner;
        let mix: Mix = to_vello_mix(blend_mode);

        let transform = self.last_transform().pre_translate(Vec2::new(
            image.width as f64 * -0.5,
            image.height as f64 * -0.5,
        ));
        let rect = Rect::new(0.0, 0.0, image.width as f64, image.height as f64);

        let builder = &mut self.scene;

        let skip_blending = mix == Mix::Normal && opacity == 1.0;

        if skip_blending {
            builder.push_layer(mix, opacity, transform, &rect);
        }

        builder.draw_image(image, transform);

        if skip_blending {
            builder.pop_layer();
        }
    }

    #[inline]
    fn draw_image_mesh(
        &mut self,
        image: &Self::Image,
        vertices: &Self::Buffer,
        uvs: &Self::Buffer,
        indices: &Self::Buffer,
        blend_mode: renderer::BlendMode,
        opacity: f32,
    ) {
        let image = &image.inner;
        let vertices = vertices.as_f32_pairs();
        let uvs = uvs.as_f32_pairs();

        let mix: Mix = to_vello_mix(blend_mode);

        for triangle_indices in indices.as_u16s().chunks_exact(3) {
            let points = [
                vertices[triangle_indices[0] as usize],
                vertices[triangle_indices[1] as usize],
                vertices[triangle_indices[2] as usize],
            ];
            let uvs = [
                uvs[triangle_indices[0] as usize],
                uvs[triangle_indices[1] as usize],
                uvs[triangle_indices[2] as usize],
            ];

            let center = Point::new(
                ((points[0][0] + points[1][0] + points[2][0]) / 3.0) as f64,
                ((points[0][1] + points[1][1] + points[2][1]) / 3.0) as f64,
            );

            let path = triangle_path(points.map(|v| Point::new(v[0] as f64, v[1] as f64)));

            let transform = self.last_transform().pre_scale_from_origin(1.03, center);
            let brush_transform =
                util::map_uvs_to_triangle(&points, &uvs, image.width, image.height);

            let builder = &mut self.scene;

            let skip_blending = mix == Mix::Normal;

            if !skip_blending {
                builder.push_layer(mix, opacity, transform, &path.bounding_box());
            }

            builder.fill(
                Fill::NonZero,
                transform,
                BrushRef::Image(image),
                Some(brush_transform),
                &path,
            );

            if !skip_blending {
                builder.pop_layer();
            }
        }
    }
}

impl fmt::Debug for Renderer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Renderer")
            .field("transforms", &self.transforms)
            .field("clips", &self.clips)
            .finish()
    }
}
