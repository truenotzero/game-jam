use core::fmt;
use std::ops::{Add, Index, IndexMut, Mul, Neg, Sub};

use crate::common::{as_bytes, Error, Result};

fn f32_eq_tolerance(lhs: f32, rhs: f32, tolerance: f32) -> bool {
    let delta = lhs - rhs;
    -tolerance < delta && delta < tolerance
}

pub fn f32_eq(lhs: f32, rhs: f32) -> bool {
    f32_eq_tolerance(lhs, rhs, 0.01)
}

// https://registry.khronos.org/OpenGL-Refpages/gl4/html/glTexImage2D.xhtml
fn srgb_to_linear(c: f32) -> f32 {
    if c < 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

as_bytes!(Vec2);

impl Vec2 {
    pub const UP: Vec2 = Self { x: 0.0f32, y: 1.0f32 };
    pub const DOWN: Vec2 = Self { x: 0.0f32, y: -1.0f32 };
    pub const LEFT: Vec2 = Self { x: -1.0f32, y: 0.0f32 };
    pub const RIGHT: Vec2 = Self { x: 1.0f32, y: 0.0f32 };

    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn diagonal(n: f32) -> Self {
        Self::new(n, n)
    }

    pub fn eq(self, rhs: Self) -> bool {
        f32_eq(self.x, rhs.x) && f32_eq(self.y, rhs.y)
    }

    pub fn abs(self) -> Self {
        Self::new(self.x.abs(), self.y.abs())
    }

    pub fn floor(self) -> Self {
        Self::new(self.x.floor(), self.y.floor())
    }
}

impl PartialEq for Vec2 {
    fn eq(&self, other: &Self) -> bool {
        Vec2::eq(*self, *other)
    }
}

impl Add for Vec2 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self
    }
}

impl Sub for Vec2 {
    type Output = Self;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self
    }
}

impl Mul<Vec2> for f32 {
    type Output = Vec2;

    fn mul(self, mut rhs: Vec2) -> Self::Output {
        rhs.x *= self;
        rhs.y *= self;
        rhs
    }
}

impl Neg for Vec2 {
    type Output = Vec2;

    fn neg(self) -> Self::Output {
        -1.0 * self
    }
}

impl From<(f32, f32)> for Vec2 {
    fn from(value: (f32, f32)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<f32> for Vec2 {
    fn from(value: f32) -> Self {
        Self::diagonal(value)
    }
}

impl From<Vec3> for Vec2 {
    fn from(value: Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

as_bytes!(Vec3);

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    fn norm(n: u8) -> f32 {
        n as f32 / 255.0
    }

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(Self::norm(r), Self::norm(g), Self::norm(b))
    }

    pub fn hexcode(hex: &str) -> Result<Self> {
        let r = u8::from_str_radix(&hex[0..2], 0x10).map_err(|_| Error::ParseError)?;
        let g = u8::from_str_radix(&hex[2..4], 0x10).map_err(|_| Error::ParseError)?;
        let b = u8::from_str_radix(&hex[4..6], 0x10).map_err(|_| Error::ParseError)?;
        Ok(Self::rgb(r, g, b))
    }

    pub fn diagonal(n: f32) -> Self {
        Self::new(n, n, n)
    }

    pub fn eq(self, rhs: Self) -> bool {
        f32_eq(self.x, rhs.x) && f32_eq(self.y, rhs.y) && f32_eq(self.z, rhs.z)
    }

    pub fn srgb_to_linear(self) -> Self {
        Self::new(
            srgb_to_linear(self.x),
            srgb_to_linear(self.y),
            srgb_to_linear(self.z),
        )
    }

    pub fn len2(self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn len(self) -> f32 {
        self.len2().sqrt()
    }

    pub fn normalize(self) -> Self {
        let s = 1.0 / self.len();
        s * self
    }
}

impl From<(f32, f32, f32)> for Vec3 {
    fn from(value: (f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2)
    }
}

impl From<f32> for Vec3 {
    fn from(value: f32) -> Self {
        Self::diagonal(value)
    }
}

impl From<(Vec2, f32)> for Vec3 {
    fn from(value: (Vec2, f32)) -> Self {
        Self {
            x: value.0.x,
            y: value.0.y,
            z: value.1,
        }
    }
}

impl From<Vec4> for Vec3 {
    fn from(value: Vec4) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}

impl Mul<Vec3> for f32 {
    type Output = Vec3;

    fn mul(self, mut rhs: Vec3) -> Self::Output {
        rhs.x *= self;
        rhs.y *= self;
        rhs.z *= self;

        rhs
    }
}

impl Add for Vec3 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;

        self
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;

        self
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

as_bytes!(Vec4);

impl Index<usize> for Vec4 {
    type Output = f32;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.w,
            _ => panic!("index should be [0,4]"),
        }
    }
}

impl IndexMut<usize> for Vec4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.x,
            1 => &mut self.y,
            2 => &mut self.z,
            3 => &mut self.w,
            _ => panic!("index should be [0,4]"),
        }
    }
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn diagonal(n: f32) -> Self {
        Self::new(n, n, n, n)
    }

    pub fn direction(direction: Vec2) -> Self {
        Self {
            x: direction.x,
            y: direction.y,
            z: 0.0,
            w: 0.0,
        }
    }

    pub fn position(position: Vec3) -> Self {
        Self {
            x: position.x,
            y: position.y,
            z: position.z,
            w: 1.0,
        }
    }
}

impl Mul<Vec4> for f32 {
    type Output = Vec4;

    fn mul(self, mut rhs: Vec4) -> Self::Output {
        rhs.x *= self;
        rhs.y *= self;
        rhs.z *= self;
        rhs.w *= self;
        rhs
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}

impl From<[f32; 4]> for Vec4 {
    fn from(value: [f32; 4]) -> Self {
        Self::new(value[0], value[1], value[2], value[3])
    }
}

impl From<f32> for Vec4 {
    fn from(value: f32) -> Self {
        Self::diagonal(value)
    }
}

// pub struct Mat3 {
//     pub xy: [[f32; 3]; 3],
// }

// impl Default for Mat3 {
//     fn default() -> Self {
//         Self::identity()
//     }
// }

// impl Mat3 {
//     pub fn zero() -> Self {
//         Self {
//             xy: Default::default(),
//         }
//     }

//     pub fn identity() -> Self {
//         let mut ret = Self::zero();
//         for i in 0..3 {
//             ret.xy[i][i] = 1.0;
//         }
//         ret
//     }

//     pub fn scale(scale: Vec2) -> Self {
//         let mut ret = Self::zero();
//         ret.xy[0][0] = scale.x;
//         ret.xy[1][1] = scale.y;
//         ret.xy[2][2] = 1.0;
//         ret
//     }

//     pub fn rotate(angle: f32) -> Self {
//         todo!()
//     }

//     pub fn translate(translate: Vec3) -> Self {
//         todo!()
//     }

// }

#[derive(Clone, Copy, Debug)]
pub struct Mat4 {
    //pub xy: [[f32; 4]; 4],
    c: [Vec4; 4],
}

as_bytes!(Mat4);

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Index<usize> for Mat4 {
    type Output = Vec4;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.c[0],
            1 => &self.c[1],
            2 => &self.c[2],
            3 => &self.c[3],
            _ => panic!("index should be [0,4]"),
        }
    }
}

impl IndexMut<usize> for Mat4 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.c[0],
            1 => &mut self.c[1],
            2 => &mut self.c[2],
            3 => &mut self.c[3],
            _ => panic!("index should be [0,4]"),
        }
    }
}

impl Mat4 {
    pub fn zero() -> Self {
        Self {
            c: Default::default(),
        }
    }

    pub fn identity() -> Self {
        let mut ret = Self::zero();
        for i in 0..4 {
            ret[i][i] = 1.0;
        }
        ret
    }

    pub fn scale(scale: Vec2) -> Self {
        let mut ret = Self::identity();
        ret[0][0] = scale.x;
        ret[1][1] = scale.y;
        ret
    }

    pub fn rotate(angle: f32) -> Self {
        todo!()
    }

    pub fn translate(translate: Vec3) -> Self {
        let mut ret = Self::identity();
        ret[3][0] = translate.x;
        ret[3][1] = translate.y;
        ret[3][2] = translate.z;
        ret
    }

    pub fn depth(depth: f32) -> Self {
        let mut ret = Self::identity();
        ret[3][2] = depth;
        ret
    }

    pub fn flip_horizontal() -> Self {
        Self::scale((1.0, -1.0).into())
    }

    pub fn flip_vertical() -> Self {
        Self::scale((-1.0, 1.0).into())
    }

    // screen projection matrix with each tile's 0,0 being offset
    pub fn screen(position: Vec2, width: f32, height: f32) -> Self {
        let l = position.x - 0.5 * width;
        let r = position.x + 0.5 * width;
        let t = position.y - 0.5 * height;
        let b = position.y + 0.5 * height;
        let f = -1.0;
        let n = 1.0;

        Self::ortho(r, l, t, b, n, f)
    }

    fn ortho(r: f32, l: f32, t: f32, b: f32, n: f32, f: f32) -> Self {
        let mut ret = Self::identity();
        ret[0][0] = 2.0 / (r - l);
        ret[1][1] = 2.0 / (t - b);
        ret[2][2] = -2.0 / (f - n);
        ret[3][0] = (l + r) / (l - r);
        ret[3][1] = (b + t) / (b - t);
        ret[3][2] = (n + f) / (n - f);
        ret
    }

    /// Invert the matrix
    /// Assumes that det(A) != 0
    /// Also assumes that the matrix is upper-triangular
    pub fn inverse(mut self) -> Self {
        let mut ret = Self::identity();

        for e in (0..4).rev() {
            let s = self[e][e];
            self[e] = (1.0 / s) * self[e];
            ret[e] = (1.0 / s) * ret[e];
            for y in (0..e).rev() {
                ret[e][y] -= s * self[e][y];
            }
        }

        ret
    }
}

impl fmt::Display for Mat4 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for y in 0..4 {
            write!(f, "[ ")?;
            for x in 0..4 {
                write!(f, "{:.2} ", self[x][y])?;
            }
            writeln!(f, "]")?;
        }

        Ok(())
    }
}

impl Mul<Mat4> for f32 {
    type Output = Mat4;

    fn mul(self, mut rhs: Mat4) -> Self::Output {
        for y in 0..4 {
            for x in 0..4 {
                rhs[x][y] *= self;
            }
        }
        rhs
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        let rhs = [rhs.x, rhs.y, rhs.z, rhs.w];
        let mut ret = [0.0; 4];

        for y in 0..4 {
            let mut sum = 0.0;
            for e in 0..4 {
                sum += self[e][y] * rhs[e];
            }
            ret[y] = sum;
        }

        Vec4::new(ret[0], ret[1], ret[2], ret[3])
    }
}

impl Mul for Mat4 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut ret = Self::zero();
        for y in 0..4 {
            for x in 0..4 {
                let mut sum = 0.0;
                for e in 0..4 {
                    sum += self[e][y] * rhs[x][e];
                }
                ret[x][y] = sum;
            }
        }

        ret
    }
}

impl Add for Mat4 {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        for y in 0..4 {
            for x in 0..4 {
                self[x][y] += rhs[x][y]
            }
        }
        self
    }
}

pub fn lerp<T>(lhs: T, rhs: T, p: f32) -> <<f32 as Mul<T>>::Output as Add>::Output
where
    f32: Mul<T>,
    <f32 as Mul<T>>::Output: Add,
{
    (1.0 - p) * lhs + p * rhs
}

/// Make animations pleasant
/// https://easings.net/#
pub mod ease {
    use super::Vec2;

    /// cubic bezier defined by (0,0), p1, p2, (1,1)
    pub struct UnitBezier {
        p1: Vec2,
        p2: Vec2,
        approximations: Vec<Vec2>,
    }

    impl UnitBezier {
        pub fn new(p1x: f32, p1y: f32, p2x: f32, p2y: f32, num_approximations: usize) -> Self {
            let p1 = Vec2::new(p1x, p1y);
            let p2 = Vec2::new(p2x, p2y);

            let step = 1.0 / num_approximations as f32;
            let mut approximations = Vec::with_capacity(num_approximations);
            for i in 0..num_approximations {
                let t = step * i as f32;
                let b = Self::t(p1, p2, t);
                approximations.push(b);
            }

            Self {
                p1,
                p2,
                approximations,
            }
        }

        /// Calculate B(t) = (x,y)
        fn t(p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
            let p3 = Vec2::diagonal(1.0);

            (3.0 * t * t * t - 6.0 * t * t + 3.0 * t) * p1
                + (-3.0 * t * t * t + 3.0 * t * t) * p2
                + (t * t * t) * p3
        }

        /// Given a point B(t) = (x,y)
        /// approximate the y value based on x
        pub fn apply(&self, x: f32) -> f32 {
            let mut low = Vec2::default();
            for v in &self.approximations {
                if v.x < x {
                    low = *v;
                } else {
                    break;
                }
            }

            let mut high = Vec2::default();
            for v in &self.approximations {
                if v.x > x {
                    high = *v;
                } else {
                    break;
                }
            }

            // normalized x
            let n = (x - low.x) / (high.x - low.x);
            super::lerp(low.y, high.y, n)
        }
    }

    pub fn out_quart(p: f32) -> f32 {
        1.0 - (1.0 - p).powf(4.0)
    }

    pub fn out_back(p: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;

        let p = p - 1.0;
        1.0 + c3 * p * p * p + c1 * p * p
    }

    pub fn in_back(p: f32) -> f32 {
        let c1 = 1.70158;
        let c3 = c1 + 1.0;

        c3 * p * p * p - c1 * p * p
    }

    pub fn out_expo(p: f32) -> f32 {
        1.0 - 2.0f32.powf(-10.0 * p)
    }
}
