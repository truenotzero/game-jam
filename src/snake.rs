use std::{cell::Cell, collections::VecDeque, ptr::null, time::Duration};

use crate::{
    common::{AsBytes, Error}, gl::{self, ArrayBuffer, DrawContext, IndexBuffer, Shader, Uniform, Vao}, math::{Vec2, Vec4}, palette::Palette, render::quad_vertex_helper
};

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Direction {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(self) -> Direction {
        match self {
            Direction::None => Direction::None,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
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

impl TryFrom<Vec2> for Direction {
    type Error = Error;

    fn try_from(value: Vec2) -> Result<Self, Self::Error> {
        if value.eq(Vec2::default()) {
            Ok(Direction::default())
        } else if value.eq(Vec2::new(0.0, -1.0)) {
            Ok(Direction::Up)
        } else if value.eq(Vec2::new(0.0, 1.0)) {
            Ok(Direction::Down)
        } else if value.eq(Vec2::new(-1.0, 0.0)) {
            Ok(Direction::Left)
        } else if value.eq(Vec2::new(1.0, 0.0)) {
            Ok(Direction::Right)
        } else {
            Err(Error::BadDirection)
        }
    }
}

pub struct Snake<'a> {
    direction: Direction,
    body: VecDeque<Vec2>,
    grow_next_tick: bool,

    move_timer: Duration,
    move_cooldown: Duration,

    palette: Palette,
    animation: Direction,

    vao: Vao<'a>,
    vertex_data: ArrayBuffer<'a>,
    index_data: IndexBuffer<'a>,
}

impl<'a> Snake<'a> {
    pub fn new(ctx: &'a DrawContext, head: Vec2, palette: Palette) -> Self {
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
            move_cooldown: Duration::from_millis(100),

            palette,
            animation: Direction::default(),

            vao,
            vertex_data,
            index_data,
        }
    }

    pub fn grow(&mut self) {
        self.grow_next_tick = true;
    }

    pub fn handle_move(&mut self, direction: Direction) {
        if direction.opposite() != self.direction {
            self.direction = direction;
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        self.move_timer += dt;
        if self.move_timer > self.move_cooldown {
            self.move_timer -= self.move_cooldown
        } else {
            return;
        }

        if self.body.len() < 7 {
            self.grow();
        }

        let new_head = self.body[0] + self.direction.into();
        self.animation = self.direction;

        if self.grow_next_tick {
            self.grow_next_tick = false;
        } else {
            self.body.pop_back();
        }

        self.body.push_front(new_head);
    }

    pub fn draw(&self, shader: &Shader) {
        // generate snake vertices on the fly
        let pct = self.move_timer.as_secs_f32() / self.move_cooldown.as_secs_f32();
        let tail = self.body.len() - 1;
        let (vertices, indices) = quad_vertex_helper(
            -0.5,
            self.body.iter().enumerate().map(|(idx, v)| {
                let mut x = v.x;
                let mut y = v.y;
                let mut width = 1.0;
                let mut height = 1.0;
                // animate head & tail
                if idx == 0 {
                    match self.animation {
                        Direction::Up => {
                            height = pct;
                            y = y + (1.0 - pct);
                        },
                        Direction::Left => {
                            width = pct;
                            x = x + (1.0 - pct);
                        }
                        Direction::Down => height = pct,
                        Direction::Right => width = pct,
                        Direction::None => (),
                    }
                } else if idx == tail {
                    let direction = Direction::try_from(self.body[tail - 1] -self.body[tail]).expect("Cannot figure tail direction");
                    match direction {
                        Direction::Up => height -= pct,
                        Direction::Down => y += pct,
                        Direction::Left => width -= pct,
                        Direction::Right => x += pct,
                        Direction::None => (),
                    }
                }
                Vec4::new(x, y, width, height)
            }),
        );
        self.vertex_data.update(0, unsafe { vertices.as_bytes() });
        self.index_data.update(0, &indices);

        self.vao.enable();
        self.index_data.bind();
        shader.enable();
        let color = shader.locate_uniform("uColor").expect("No uColor in snake shader");
        self.palette.snake.uniform(color);
        gl::call!(DrawElements(
            TRIANGLES,
            indices.len() as _,
            UNSIGNED_BYTE,
            null()
        ));
    }
}
