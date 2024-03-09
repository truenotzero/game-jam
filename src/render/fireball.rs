use std::{
    mem::{offset_of, size_of},
    path::Path,
};

use crate::{
    common::{as_bytes, AsBytes},
    gl::{self, ArrayBuffer, DrawContext, Shader, Vao},
    math::{Vec2, Vec3},
};

pub struct Fireball {
    pub pos: Vec2,
    pub col: Vec3,
    pub radius: f32,
}

as_bytes!(Fireball);

pub struct FireballManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,

    num_fireballs: usize,
    max_fireballs: usize,
}

impl<'a> FireballManager<'a> {
    pub fn new(ctx: &'a DrawContext, max_fireballs: usize) -> Self {
        let vbo = ArrayBuffer::new(ctx);
        vbo.reserve(
            max_fireballs * size_of::<Fireball>(),
            gl::buffer_flags::DYNAMIC_STORAGE,
        );
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
            size_of::<Fireball>() as _,
            offset_of!(Fireball, pos) as _,
        ));
        // aCol
        gl::call!(EnableVertexAttribArray(1));
        gl::call!(VertexAttribPointer(
            1,
            3,
            FLOAT,
            FALSE,
            size_of::<Fireball>() as _,
            offset_of!(Fireball, col) as _,
        ));
        // aRadius
        gl::call!(EnableVertexAttribArray(2));
        gl::call!(VertexAttribPointer(
            2,
            1,
            FLOAT,
            FALSE,
            size_of::<Fireball>() as _,
            offset_of!(Fireball, radius) as _,
        ));

        let shader = Shader::from_file(ctx, Path::new("res/shaders/fireball"))
            .expect("Fireball shader error");

        Self {
            vao,
            vbo,
            shader,

            num_fireballs: 0,
            max_fireballs,
        }
    }

    pub fn push(&mut self, fireball: Fireball) {
        if self.num_fireballs == self.max_fireballs {
            panic!("max fireballs")
        }

        self.vbo
            .update(self.num_fireballs * size_of::<Fireball>(), unsafe {
                fireball.as_bytes()
            });

        self.num_fireballs += 1;
    }

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();
        gl::call!(DrawArrays(POINTS, 0, self.num_fireballs as _));

        self.num_fireballs = 0;
    }
}
