use std::ops::{Add, Mul, Sub};

use crate::common::as_bytes;

fn f32_eq_tolerance(lhs: f32, rhs: f32, tolerance: f32) -> bool {
    let delta = lhs - rhs;
    -tolerance < delta && delta < tolerance
}

pub fn f32_eq(lhs: f32, rhs: f32) -> bool {
    f32_eq_tolerance(lhs, rhs, 0.01)
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

as_bytes!(Vec2);

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn diagonal(n: f32) -> Self {
        Self::new(n, n)
    }

    pub fn eq(self, rhs: Self) -> bool {
        f32_eq(self.x, rhs.x) && f32_eq(self.y, rhs.y)
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

    pub fn diagonal(n: f32) -> Self {
        Self::new(n, n, n)
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

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct Vec4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

as_bytes!(Vec4);

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn diagonal(n: f32) -> Self {
        Self::new(n, n, n, n)
    }
}

impl From<(f32, f32, f32, f32)> for Vec4 {
    fn from(value: (f32, f32, f32, f32)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
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
    pub xy: [[f32; 4]; 4],
}

as_bytes!(Mat4);

impl Default for Mat4 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat4 {
    pub fn zero() -> Self {
        Self {
            xy: Default::default(),
        }
    }

    pub fn identity() -> Self {
        let mut ret = Self::zero();
        for i in 0..4 {
            ret.xy[i][i] = 1.0;
        }
        ret
    }

    pub fn scale(scale: Vec2) -> Self {
        let mut ret = Self::identity();
        ret.xy[0][0] = scale.x;
        ret.xy[1][1] = scale.y;
        ret
    }

    pub fn rotate(angle: f32) -> Self {
        todo!()
    }

    pub fn translate(translate: Vec3) -> Self {
        let mut ret = Self::identity();
        ret.xy[3][0] = translate.x;
        ret.xy[3][1] = translate.y;
        ret.xy[3][2] = translate.z;
        ret
    }

    pub fn depth(depth: f32) -> Self {
        let mut ret = Self::identity();
        ret.xy[3][2] = depth;
        ret
    }

    pub fn flip_horizontal() -> Self {
        Self::scale((1.0, -1.0).into())
    }

    pub fn flip_vertical() -> Self {
        Self::scale((-1.0, 1.0).into())
    }

    // screen projection matrix with each tile's 0,0 being centered
    pub fn screen_local_center(width: f32, height: f32) -> Self {
        let l = -0.5;
        let r = width - 0.5;
        let t = -0.5;
        let b = height - 0.5;
        let f = -1.0;
        let n = 1.0;

        Self::ortho(r, l, t, b, n, f)
    }

    // screen projection matrix with each tile's 0,0 being offset
    pub fn screen(width: f32, height: f32) -> Self {
        let l = 0.0;
        let r = width;
        let t = 0.0;
        let b = height;
        let f = -1.0;
        let n = 1.0;

        Self::ortho(r, l, t, b, n, f)
    }

    fn ortho(r: f32, l: f32, t: f32, b: f32, n: f32, f: f32) -> Self {
        let mut ret = Self::identity();
        ret.xy[0][0] = 2.0 / (r - l);
        ret.xy[1][1] = 2.0 / (t - b);
        ret.xy[2][2] = -2.0 / (f - n);
        ret.xy[3][0] = (l + r) / (l - r);
        ret.xy[3][1] = (b + t) / (b - t);
        ret.xy[3][2] = (n + f) / (n - f);
        ret
    }
}

impl Mul<Mat4> for f32 {
    type Output = Mat4;
    
    fn mul(self, mut rhs: Mat4) -> Self::Output {
        for y in 0..4 {
            for x in 0..4 {
                rhs.xy[x][y] *= self;
            }
        }
        rhs
    }
}

impl Mul<Vec4> for Mat4 {
    type Output = Vec4;

    fn mul(self, rhs: Vec4) -> Self::Output {
        let rhs: [f32; 4] = unsafe { std::mem::transmute(rhs) };
        let mut ret = [0f32;4];

        for y in 0..4 {
            let mut sum = 0.0;
            for e in 0..4 {
                sum += self.xy[e][y] * rhs[e];
            }
            ret[y] = sum;
        }
        
        unsafe { std::mem::transmute(ret) }
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
                    sum += self.xy[e][y] * rhs.xy[x][e];
                }
                ret.xy[x][y] = sum;
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
                self.xy[x][y] += rhs.xy[x][y]
            }
        }
        self
    }
}


pub fn lerp<T>(lhs: T, rhs: T, p: f32) -> <<f32 as Mul<T>>::Output as Add>::Output where f32: Mul<T>, <f32 as Mul<T>>::Output: Add {
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
        pub fn new(p1: Vec2, p2: Vec2, num_approximations: usize) -> Self {
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

            (  3.0 * t * t * t - 6.0 * t * t + 3.0 * t ) * p1 +
            ( -3.0 * t * t * t + 3.0 * t * t )           * p2 + 
            (        t * t * t )                         * p3
        }


        /// Given a point B(t) = (x,y)
        /// approximate the y value based on x
        pub fn x(&self, x: f32) -> f32 {
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
}
