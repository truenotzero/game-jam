use std::ops::Mul;

use crate::common::as_bytes;

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

pub struct Mat3 {
    pub xy: [[f32; 3]; 3],
}

impl Default for Mat3 {
    fn default() -> Self {
        Self::identity()
    }
}

impl Mat3 {
    pub fn zero() -> Self {
        Self {
            xy: Default::default(),
        }
    }

    pub fn identity() -> Self {
        let mut ret = Self::zero();
        for i in 0..3 {
            ret.xy[i][i] = 1.0;
        }
        ret
    }

    pub fn scale(scale: Vec2) -> Self {
        let mut ret = Self::zero();
        ret.xy[0][0] = scale.x;
        ret.xy[1][1] = scale.y;
        ret.xy[2][2] = 1.0;
        ret
    }

    pub fn rotate(angle: f32) -> Self {
        todo!()
    }

    pub fn translate(translate: Vec3) -> Self {
        todo!()
    }

}

#[derive(Clone, Copy)]
pub struct Mat4 {
    pub xy: [[f32; 4]; 4],
}

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
        let mut ret = Self::zero();
        ret.xy[0][0] = scale.x;
        ret.xy[1][1] = scale.y;
        ret.xy[2][2] = 1.0;
        ret.xy[3][3] = 1.0;
        ret
    }

    pub fn rotate(angle: f32) -> Self {
        todo!()
    }

    pub fn translate(translate: Vec2) -> Self {
        let mut ret = Self::identity();
        ret.xy[3][0] = translate.x;
        ret.xy[3][1] = translate.y;
        ret
    }

    pub fn depth(depth: f32) -> Self {
        let mut ret = Self::identity();
        ret.xy[3][2] = depth;
        ret
    }

    pub fn flip_horizontal() -> Self {
        Self::scale((0.0, -1.0).into())
    }

    pub fn flip_vertical() -> Self {
        Self::scale((-1.0, 0.0).into())
    }

    // screen projection matrix
    // sets the upper left corner as 0,0
    // sets the lower right corner as width,height
    // 
    pub fn screen(width: f32, height: f32) -> Self {
        // flip y
        let flip = Self::flip_horizontal();
        // move origin to top left
        let origin = Self::translate((-0.5, 0.5).into());
        // scale
        let scale = Mat4::scale((2.0 / width, 2.0 / height).into());

        scale * origin * flip
    }
}

impl Mul for Mat4 {
    type Output=Self;

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
