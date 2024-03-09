use std::mem::{offset_of, size_of};

use crate::{
    common::{as_bytes, AsBytes},
    gl::{self, ArrayBuffer, DrawContext, Shader, Vao},
    math::{Vec2, Vec3}, resources,
};

#[repr(C)]
pub struct Shield {
    pos: Vec2,
    col: Vec3,
    radius: f32,
    num_sides: i32,
    sides0: Vec2,
    sides1: Vec2,
    sides2: Vec2,
    sides3: Vec2,
}

impl Shield {
    pub fn new(pos: Vec2, col: Vec3, radius: f32) -> Self {
        Self {
            pos,
            col,
            radius,
            num_sides: 0,
            sides0: Default::default(),
            sides1: Default::default(),
            sides2: Default::default(),
            sides3: Default::default(),
        }
    }

    pub fn push_side(mut self, side: Vec2) -> Self {
        match self.num_sides {
            0 => self.sides0 = side,
            1 => self.sides1 = side,
            2 => self.sides2 = side,
            3 => self.sides3 = side,
            _ => panic!(),
        }

        self.num_sides += 1;
        self
    }
}

as_bytes!(Shield);

struct VaoHelper<'a> {
    vao: Vao<'a>,
    attrib: gl::raw::GLuint,
}

impl<'a> VaoHelper<'a> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let vao = Vao::new(ctx);
        vao.apply();
        Self { vao, attrib: 0 }
    }

    pub fn bind_buffer(self, buf: &ArrayBuffer) -> Self {
        buf.apply();
        self
    }

    pub fn push_attrib(
        mut self,
        size: gl::raw::GLint,
        type_: gl::raw::GLenum,
        normalized: gl::raw::GLboolean,
        stride: usize,
        pointer: usize,
    ) -> Self {
        gl::call!(EnableVertexAttribArray(self.attrib));
        gl::call!(VertexAttribPointer(
            self.attrib,
            size,
            type_,
            normalized,
            stride as _,
            pointer as _
        ));

        // println!(
        //     "[{}] Attribute=[size:{},type:{},normalized:{},stride:{},pointer:{}]",
        //     self.attrib, size, type_, normalized, stride, pointer
        // );

        self.attrib += 1;
        self
    }

    pub fn push_int_attrib(
        mut self,
        size: gl::raw::GLint,
        type_: gl::raw::GLenum,
        stride: usize,
        pointer: usize,
    ) -> Self {
        gl::call!(EnableVertexAttribArray(self.attrib));
        gl::call!(VertexAttribIPointer(
            self.attrib,
            size,
            type_,
            stride as _,
            pointer as _
        ));

        // println!(
        //     "[{}] Int Attribute=[size:{},type:{},stride:{},pointer:{}]",
        //     self.attrib, size, type_, stride, pointer
        // );

        self.attrib += 1;
        self
    }

    pub fn build(self) -> Vao<'a> {
        self.vao
    }
}

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
        vbo.reserve(
            max_shields * size_of::<Shield>(),
            gl::buffer_flags::DYNAMIC_STORAGE,
        );
        vbo.apply();

        let vao = VaoHelper::new(ctx)
            .bind_buffer(&vbo)
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Shield>(),
                offset_of!(Shield, pos),
            )
            .push_attrib(
                3,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Shield>(),
                offset_of!(Shield, col),
            )
            .push_attrib(
                1,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Shield>(),
                offset_of!(Shield, radius),
            )
            .push_int_attrib(
                1,
                gl::raw::INT,
                size_of::<Shield>(),
                offset_of!(Shield, num_sides),
            )
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Shield>(),
                offset_of!(Shield, sides0),
            )
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Shield>(),
                offset_of!(Shield, sides1),
            )
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Shield>(),
                offset_of!(Shield, sides2),
            )
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Shield>(),
                offset_of!(Shield, sides3),
            );

        // let shader = Shader::from_file(ctx, Path::new("res/shaders/shield")).unwrap();
        let shader = Shader::from_resource(ctx, resources::shaders::SHIELD).unwrap();
        Self {
            vao: vao.build(),
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

        self.vbo
            .update(self.num_shields * size_of::<Shield>(), unsafe {
                shield.as_bytes()
            });

        self.num_shields += 1;
    }

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();
        gl::call!(DrawArrays(POINTS, 0, self.num_shields as _));

        self.num_shields = 0;
    }
}
