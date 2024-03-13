use std::mem::{offset_of, size_of};

use crate::{
    common::{as_bytes, AsBytes},
    gl::{self, ArrayBuffer, DrawContext, Shader, Vao},
    math::{Vec2, Vec4},
    resources,
};

use super::VaoHelper;

#[repr(C)]
pub struct Shield {
    pos: Vec2,
    col: Vec4,
    radius: f32,
    is_fix: u8,
    num_sides: u8,
    sides0: Vec2,
    sides1: Vec2,
    sides2: Vec2,
    sides3: Vec2,
}

impl Shield {
    pub fn new(pos: Vec2, col: Vec4, is_fix: bool, radius: f32) -> Self {
        Self {
            pos,
            col,
            radius,
            is_fix: if is_fix { 1 } else { 0 },
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

    pub fn push_quad(self) -> Self {
        self
            .push_side(Vec2::UP)
            .push_side(Vec2::DOWN)
            .push_side(Vec2::LEFT)
            .push_side(Vec2::RIGHT)
    }
}

as_bytes!(Shield);

pub struct ShieldManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,

    max_shields: usize,
    shields: Vec<Shield>,
    fixes: Vec<Shield>,
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
                4,
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
                gl::raw::BYTE,
                size_of::<Shield>(),
                offset_of!(Shield, is_fix),
            )
            .push_int_attrib(
                1,
                gl::raw::BYTE,
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
        let shader = Shader::from_resource(ctx, resources::shaders::SHIELD)
            .expect("shield shader should compile properly");
        Self {
            vao: vao.build(),
            vbo,
            shader,

            max_shields,

            shields: Vec::new(),
            fixes: Vec::new(),
        }
    }

    fn update_buffer(&mut self, is_fixes: bool) -> gl::raw::GLint {
        let buf = if is_fixes {
            &mut self.fixes
        } else {
            &mut self.shields
        };
        for (idx, shield) in buf.iter_mut().enumerate() {
            self.vbo
                .update(idx * size_of::<Shield>(), unsafe { shield.as_bytes() });
        }

        let len = buf.len() as _;
        buf.clear();

        len
    }

    pub fn push(&mut self, shield: Shield) {
        if shield.is_fix == 1 {
            self.fixes.push(shield);
        } else {
            self.shields.push(shield);
        }
    }

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();

        let fixes = self.update_buffer(true);
        if fixes > 0 {
            gl::call!(DrawArrays(POINTS, 0, fixes));
        }

        let shields = self.update_buffer(false);
        if shields > 0 {
            gl::call!(DrawArrays(POINTS, 0, shields));
        }
    }
}
