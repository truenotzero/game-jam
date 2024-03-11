use std::mem::{offset_of, size_of};

use crate::{
    common::{as_bytes, AsBytes},
    gl::{self, ArrayBuffer, DrawContext, Shader, Vao},
    math::{Mat4, Vec2}, resources,
};

use super::VaoHelper;

#[derive(Debug, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

as_bytes!(Vertex);

#[derive(Debug)]
pub struct Text {
    vertices: [Vertex; 6],
}

impl Default for Text {
    fn default() -> Text {
        let corners = [
            Vertex {
                pos: Vec2::new(-0.5, 0.5),
                uv: Vec2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vec2::new(-0.5, -0.5),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vec2::new(0.5, 0.5),
                uv: Vec2::new(1.0, 1.0),
            },
            Vertex {
                pos: Vec2::new(0.5, -0.5),
                uv: Vec2::new(1.0, 0.0),
            },
        ];

        Self {
            vertices: [
                corners[0], corners[1], corners[2], corners[3], corners[2], corners[1],
            ],
        }
    }
}

impl Text {
    pub fn new<D: Into<Vec2>>(pos: Vec2, scale: f32, direction: D) -> Self {
        Self::default()
            .transform(Mat4::scale(scale.into()))
            .transform(Mat4::translate((pos, 0.0).into()))
    }

    pub fn transform(mut self, t: Mat4) -> Self {
        for v in &mut self.vertices {
            v.pos = t * v.pos;
        }

        self
    }
}

pub struct TextManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,

    num_texts: usize,
    max_texts: usize,
}

impl<'a> TextManager<'a> {
    pub fn new(ctx: &'a DrawContext, max_texts: usize) -> Self {
        let max_texts = max_texts * size_of::<Text>();
        let vbo = ArrayBuffer::new(ctx);
        vbo.reserve(max_texts, gl::buffer_flags::DYNAMIC_STORAGE);

        let vao = VaoHelper::new(ctx)
            .bind_buffer(&vbo)
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Vertex>(),
                offset_of!(Vertex, pos),
            )
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Vertex>(),
                offset_of!(Vertex, uv),
            )
            .build();

        Self {
            vao,
            vbo,
            shader: Shader::from_resource(ctx, resources::shaders::TEXT)
                .expect("bad text shader"),

            num_texts: 0,
            max_texts,
        }
    }

    pub fn push(&mut self, text: Text) {
        for v in text.vertices {
            if self.num_texts == self.max_texts {
                panic!("max texts")
            }

            self.vbo.update(self.num_texts * size_of::<Vertex>(), unsafe { v.as_bytes() });

            self.num_texts += 1;
        }
    }

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();
        gl::call!(DrawArrays(TRIANGLES, 0, self.num_texts as _));

        self.num_texts = 0;
    }
}
