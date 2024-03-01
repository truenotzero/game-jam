use crate::common::as_bytes;

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
}
