use std::ops::{Add, Mul, Sub};

use vello::kurbo::{Affine, Point};

#[derive(Clone, Copy, Debug)]
struct Vec2 {
    x: f32,
    y: f32,
}

impl Add for Vec2 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul<f32> for Vec2 {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// Finds the affine transform that maps triangle `from` to triangle `to`. The algorithm is based
/// on the [Simplex Affine Mapping] method which has a [Swift implementation].
///
/// [Simplex Affine Mapping]: https://www.researchgate.net/publication/332410209_Beginner%27s_guide_to_mapping_simplexes_affinely
/// [Swift implementation]: https://rethunk.medium.com/finding-an-affine-transform-using-three-2d-point-correspondences-using-simplex-affine-mapping-255aeb4e8055
fn simplex_affine_mapping(from: [Vec2; 3], to: [Vec2; 3]) -> Affine {
    let [a, b, c] = from;
    let [d, e, f] = to;

    let det_recip = (a.x * b.y + b.x * c.y + c.x * a.y - a.x * c.y - b.x * a.y - c.x * b.y).recip();

    let p = (d * (b.y - c.y) - e * (a.y - c.y) + f * (a.y - b.y)) * det_recip;

    let q = (e * (a.x - c.x) - d * (b.x - c.x) - f * (a.x - b.x)) * det_recip;

    let t = (d * (b.x * c.y - b.y * c.x) - e * (a.x * c.y - a.y * c.x)
        + f * (a.x * b.y - a.y * b.x))
        * det_recip;

    Affine::new([
        p.x as f64, p.y as f64, q.x as f64, q.y as f64, t.x as f64, t.y as f64,
    ])
}

pub fn map_uvs_to_triangle(
    points: &[[f32; 2]; 3],
    uvs: &[[f32; 2]; 3],
    width: u32,
    height: u32,
) -> Affine {
    simplex_affine_mapping(
        uvs.map(|v| Vec2 {
            x: v[0] * width as f32,
            y: v[1] * height as f32,
        }),
        points.map(|v| Vec2 { x: v[0], y: v[1] }),
    )
}

pub trait ScaleFromOrigin {
    fn pre_scale_from_origin(self, scale: f64, origin: Point) -> Self;
}

impl ScaleFromOrigin for Affine {
    fn pre_scale_from_origin(self, scale: f64, origin: Point) -> Self {
        let origin = origin.to_vec2();
        self.pre_translate(origin)
            .pre_scale(scale)
            .pre_translate(-origin)
    }
}
