use std::{cell::Cell, collections::VecDeque, ptr::null, time::Duration};

use crate::{common::AsBytes, gl::{self, ArrayBuffer, DrawContext, IndexBuffer, Shader, Vao}, math::{Vec2, Vec4}, render::quad_vertex_helper};

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Direction {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

impl From<Direction> for Vec2 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::None => Self::default(),
            Direction::Up => Self::new(0.0, -1.0),
            Direction::Down => Self::new(0.0, 1.0),
            Direction::Left => Self::new(-1.0, 0.0),
            Direction::Right => Self::new(1.0, 0.0),
        }
    }
}

pub struct Snake<'a> {
    direction: Direction,
    body: VecDeque<Vec2>,
    grow_next_tick: bool,

    move_timer: Duration,

    // cache values, which are logically immutable, while practically being mutable
    // to speed up rendering
    // but then again it's 2d lol
    generate_draw_data: Cell<bool>, 
    num_indices: Cell<gl::raw::GLsizei>,

    vao: Vao<'a>,
    vertex_data: ArrayBuffer<'a>,
    index_data: IndexBuffer<'a>,
}

impl<'a> Snake<'a> {
    pub fn new(ctx: &'a DrawContext, head: Vec2) -> Self {
        let body = VecDeque::from([head]);

        let vao = Vao::new(ctx);
        let vertex_data = ArrayBuffer::new(ctx);
        let index_data = IndexBuffer::new(ctx);

        vertex_data.reserve(1024, gl::buffer_flags::DYNAMIC_STORAGE);
        index_data.reserve(256, gl::buffer_flags::DYNAMIC_STORAGE);
        vao.bind_vertex_attribs(&vertex_data);

        Self {
            direction: Direction::default(),
            body,
            grow_next_tick: false,

            move_timer: Duration::ZERO,

            generate_draw_data: Cell::new(true),
            num_indices: Cell::default(),

            vao,
            vertex_data,
            index_data,
        }
    }

    pub fn grow(&mut self) {
        self.grow_next_tick = true;
    }

    pub fn handle_move(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn tick(&mut self, dt: Duration) {
        let move_cooldown = Duration::from_millis(100);

        self.move_timer += dt;
        if self.move_timer > move_cooldown {
            self.move_timer -= move_cooldown
        } else {
            return;
        }

        if self.body.len() < 5 {
            self.grow();
        }

        let new_head = self.body[0] + self.direction.into();

        if self.grow_next_tick {
            self.grow_next_tick = false;
            self.generate_draw_data.replace(true);
        } else {
            self.body.pop_back();
        }

        self.body.push_front(new_head);
    }

    pub fn draw(&self, shader: &Shader) {
        // generate snake vertices on the fly
        // take gets the value and puts a default (false) in the cell
        // if self.generate_draw_data.take() {
            let width = 1.0;
            let height = 1.0;
            let (vertices, indices) = quad_vertex_helper(-0.5, self.body.iter().map(|v| Vec4::new(v.x, v.y, width, height)));
            self.vertex_data.update(0, unsafe { vertices.as_bytes() });
            self.index_data.update(0, &indices);

            let pct = self.move_timer.as_secs_f32();
            let anticipate = pct * Vec2::from(self.direction);

            self.num_indices.replace(indices.len() as _);
        // }
        
        self.vao.enable();
        self.index_data.bind();
        shader.enable();
        gl::call!(DrawElements(TRIANGLES, self.num_indices.get(), UNSIGNED_BYTE, null()));
    }
}
