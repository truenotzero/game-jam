use std::{mem::{offset_of, size_of}, path::Path};

use crate::{common::{as_bytes, AsBytes}, gl::{self, ArrayBuffer, DrawContext, Shader, Vao}, math::{Vec2, Vec3}};


pub struct Shield {
    pub pos: Vec3,
    pub col: Vec3,
    pub radius: f32,
}

as_bytes!(Shield);

pub struct ShieldManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,

    num_shields: usize,
    max_shields: usize,
}

impl<'a> ShieldManager<'a> {
    pub fn new(ctx: &'a DrawContext, max_shields: usize) -> Self {
        let vbo = ArrayBuffer::new(ctx);
        vbo.reserve(max_shields * size_of::<Shield>(), gl::buffer_flags::DYNAMIC_STORAGE);
        vbo.apply();

        let vao = Vao::new(ctx);
        vao.apply();
        // aPos
        gl::call!(EnableVertexAttribArray(0));
        gl::call!(VertexAttribPointer(
            0,
            2,
            FLOAT,
            FALSE,
            size_of::<Shield>() as _,
            offset_of!(Shield, pos) as _,
        ));
        // aCol
        gl::call!(EnableVertexAttribArray(1));
        gl::call!(VertexAttribPointer(
            1,
            3,
            FLOAT,
            FALSE,
            size_of::<Shield>() as _,
            offset_of!(Shield, col) as _,
        ));
        // aRadius
        gl::call!(EnableVertexAttribArray(2));
        gl::call!(VertexAttribPointer(
            2,
            1,
            FLOAT,
            FALSE,
            size_of::<Shield>() as _,
            offset_of!(Shield, radius) as _,
        ));

        let shader = Shader::from_file(ctx, Path::new("res/shaders/shield")).unwrap();
        Self {
            vao,
            vbo,
            shader,

            num_shields: 0,
            max_shields,
        }
    }
    
    pub fn push(&mut self, shield: Shield) {
        if self.num_shields == self.max_shields {
            panic!("max shields")
        }

        self.vbo.update(self.num_shields * size_of::<Shield>(), unsafe { shield.as_bytes() });

        self.num_shields += 1;
    }
    
    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();
        gl::call!(DrawArrays(POINTS, 0, self.num_shields as _));

        self.num_shields = 0;
    }
}
