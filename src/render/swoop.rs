use std::mem::{offset_of, size_of};

use crate::{
    common::{as_bytes, AsBytes},
    entity::Direction,
    gl::{self, buffer_flags, ArrayBuffer, DrawContext, Shader, Vao},
    math::{Mat4, Vec2},
    resources,
};

use super::VaoHelper;

#[repr(C)]
#[derive(Debug)]
struct SwoopVertex {
    pos: Vec2,
    uv: Vec2,
}

as_bytes!(SwoopVertex);

#[derive(Debug)]
pub struct Swoop {
    vertices: [SwoopVertex; 4],
}

impl Default for Swoop {
    fn default() -> Swoop {
        Self {
            vertices: [
                SwoopVertex {
                    pos: Vec2::new(-0.5, 0.5),
                    uv: Vec2::new(0.0, 0.0),
                },
                SwoopVertex {
                    pos: Vec2::new(-0.5, -0.5),
                    uv: Vec2::new(0.0, 1.0),
                },
                SwoopVertex {
                    pos: Vec2::new(0.5, 0.5),
                    uv: Vec2::new(1.0, 0.0),
                },
                SwoopVertex {
                    pos: Vec2::new(0.5, -0.5),
                    uv: Vec2::new(1.0, 1.0),
                },
            ],
        }
    }
}

impl Swoop {
    pub fn new<D: Into<Vec2>>(pos: Vec2, scale: f32, direction: D) -> Self {
        Self::default()
            //.transform(Mat4::translate((pos, 0.0).into()))
            //.transform(Mat4::scale(scale.into()))
            //.transform(Mat4::rotate(Into::<Vec2>::into(direction).angle()))
    }

    pub fn transform(mut self, t: Mat4) -> Self {
        for v in &mut self.vertices {
            v.pos = t * v.pos;
            v.uv = t * v.uv;
        }

        self
    }
}

pub struct SwoopManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,

    num_swoops: usize,
    max_swoops: usize,
}

impl<'a> SwoopManager<'a> {
    pub fn new(ctx: &'a DrawContext, max_swoops: usize) -> Self {
        let vbo = ArrayBuffer::new(ctx);
        vbo.reserve(
            max_swoops * size_of::<SwoopVertex>(),
            gl::buffer_flags::DYNAMIC_STORAGE,
        );

        let vao = VaoHelper::new(ctx)
            .bind_buffer(&vbo)
            .push_attrib(2, gl::raw::FLOAT, gl::raw::FALSE, size_of::<SwoopVertex>(), offset_of!(SwoopVertex, pos))
            .push_attrib(2, gl::raw::FLOAT, gl::raw::FALSE, size_of::<SwoopVertex>(), offset_of!(SwoopVertex, uv))
            .build();

        Self {
            vao,
            vbo,
            shader: Shader::from_resource(ctx, resources::shaders::SWOOP)
                .expect("bad swoop shader"),

            num_swoops: 0,
            max_swoops,
        }
    }

    pub fn push(&mut self, swoop: Swoop) {
        println!("pushing swoop: {swoop:?}");
        for v in swoop.vertices {
            if self.num_swoops == self.max_swoops {
                panic!("max swoops")
            }

            self.vbo
                .update(self.num_swoops * size_of::<SwoopVertex>(), unsafe {
                    v.as_bytes()
                });

            self.num_swoops += 1;
        }
    }

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();
        gl::call!(DrawArrays(TRIANGLE_STRIP, 0, self.num_swoops as _));

        self.num_swoops = 0;
    }
}
