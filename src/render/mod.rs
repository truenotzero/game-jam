use core::slice;
use std::{collections::HashMap, mem::size_of_val, ptr::null, time::{Duration, Instant}};

use crate::{gl::{self, call, ArrayBuffer, DrawContext, FrameBuffer, Shader, Uniform, Vao}, math::Vec4, resources};

use self::{
    fireball::{Fireball, FireballManager},
    instanced::{quad_vertex_helper, quad_vertex_helper_local_center, InstancedShapeManager, Tile},
    shield::{Shield, ShieldManager},
};

pub mod fireball;
pub mod instanced;
pub mod shield;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum RenderType {
    Tile,
    Fireball,
    Shield,
}

pub enum Element {
    Tile(Tile),
    Fireball(Fireball),
    Shield(Shield),
}

impl From<Tile> for Element {
    fn from(value: Tile) -> Self {
        Self::Tile(value)
    }
}

impl From<Fireball> for Element {
    fn from(value: Fireball) -> Self {
        Self::Fireball(value)
    }
}

impl From<Shield> for Element {
    fn from(value: Shield) -> Self {
        Self::Shield(value)
    }
}

pub enum Renderer<'a> {
    Tile(InstancedShapeManager<'a>),
    Fireball(FireballManager<'a>),
    Shield(ShieldManager<'a>),
}

impl<'a> From<InstancedShapeManager<'a>> for Renderer<'a> {
    fn from(value: InstancedShapeManager<'a>) -> Self {
        Self::Tile(value)
    }
}

impl<'a> From<FireballManager<'a>> for Renderer<'a> {
    fn from(value: FireballManager<'a>) -> Self {
        Self::Fireball(value)
    }
}

impl<'a> From<ShieldManager<'a>> for Renderer<'a> {
    fn from(value: ShieldManager<'a>) -> Self {
        Self::Shield(value)
    }
}

impl<'a> Renderer<'a> {
    fn render_type(&self) -> RenderType {
        match self {
            Renderer::Tile(_) => RenderType::Tile,
            Renderer::Fireball(_) => RenderType::Fireball,
            Renderer::Shield(_) => RenderType::Shield,
        }
    }

    pub fn push(&mut self, element: impl Into<Element>) {
        let element = element.into();
        match self {
            Renderer::Tile(tile) => {
                if let Element::Tile(t) = element {
                    tile.push(t)
                }
            }
            Renderer::Fireball(fire) => {
                if let Element::Fireball(f) = element {
                    fire.push(f)
                }
            }
            Renderer::Shield(shield) => {
                if let Element::Shield(s) = element {
                    shield.push(s)
                }
            }
        }
    }

    pub fn draw(&mut self) {
        match self {
            Renderer::Tile(t) => t.draw(),
            Renderer::Fireball(f) => f.draw(),
            Renderer::Shield(s) => s.draw(),
        }
    }
}

pub struct RenderManager<'a> {
    framebuffer: FrameBuffer<'a>,
    vao: Vao<'a>,
    _vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,
    start_time: Instant,

    renderers: HashMap<RenderType, Renderer<'a>>,
}

impl<'a> RenderManager<'a> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let vao = Vao::new(ctx);
        let vbo = ArrayBuffer::new(ctx);
        let vertex_positions = [
            // positions    uv coords
            -1.0,  1.0,     0.0, 1.0,
             1.0,  1.0,     1.0, 1.0,
            -1.0, -1.0,     0.0, 0.0,
             1.0, -1.0,     1.0, 0.0f32,
        ];
        // safe because it's just reintrepreting the data to send to the GPU
        // cbb to waste more time on pointless abstractions I won't reuse
        let black_magic = {
            let len = size_of_val(&vertex_positions);
            let ptr = vertex_positions.as_ptr().cast();
            unsafe { slice::from_raw_parts(ptr, len) }
        };
        vbo.set(black_magic, gl::buffer_flags::DEFAULT);
        vbo.apply();
        vao.apply();
        gl::call!(EnableVertexAttribArray(0));
        gl::call!(VertexAttribPointer(0, 2, FLOAT, FALSE, 4 * 4, (4 * 0) as _));
        gl::call!(EnableVertexAttribArray(1));
        gl::call!(VertexAttribPointer(1, 2, FLOAT, FALSE, 4 * 4, (4 * 2) as _));

        Self {
            framebuffer: FrameBuffer::new_screen(ctx),
            vao,
            _vbo: vbo,
            shader: Shader::from_resource(ctx, resources::shaders::CRT).expect("bad crt shader"),
            start_time: Instant::now(),

            renderers: Default::default(),
        }
    }

    pub fn add_renderer(&mut self, renderer: impl Into<Renderer<'a>>) {
        let renderer = renderer.into();
        self.renderers.insert(renderer.render_type(), renderer);
    }

    pub fn push(&mut self, element: impl Into<Element>) {
        match element.into() {
            Element::Tile(tile) => self
                .renderers
                .get_mut(&RenderType::Tile)
                .map(|r| r.push(tile)),
            Element::Fireball(fire) => self
                .renderers
                .get_mut(&RenderType::Fireball)
                .map(|r| r.push(fire)),
            Element::Shield(shield) => self
                .renderers
                .get_mut(&RenderType::Shield)
                .map(|r| r.push(shield)),
        };
    }

    pub fn draw(&mut self) {
        // render the scene first
        // for transparency to work properly
        // render back to front
        self.framebuffer.with(|_| {
            FrameBuffer::clear();

            self.renderers.get_mut(&RenderType::Tile).map(|r| r.draw());
            self.renderers
                .get_mut(&RenderType::Shield)
                .map(|r| r.draw());
            self.renderers
                .get_mut(&RenderType::Fireball)
                .map(|r| r.draw());
        });

        // render the texture onto the monitor
        FrameBuffer::clear();
        self.vao.apply();
        self.shader.apply();
        self.start_time.elapsed().as_millis().uniform(0);
        self.framebuffer.bind_texture(0);
        call!(DrawArrays(TRIANGLE_STRIP, 0, 4));
    }
}
