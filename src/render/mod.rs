use core::slice;
use std::{
    collections::HashMap,
    mem::size_of_val,
    time::{Duration, Instant},
};

use crate::{
    gl::{self, call, ArrayBuffer, DrawContext, FrameBuffer, Shader, Uniform, Vao},
    math::{ease, Vec3},
    resources,
};

use self::{
    fireball::{Fireball, FireballManager},
    instanced::{InstancedShapeManager, Tile},
    shield::{Shield, ShieldManager},
    swoop::{Swoop, SwoopManager},
    text::{Text, TextManager},
};

pub mod fireball;
pub mod instanced;
pub mod shield;
pub mod swoop;
pub mod text;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
enum RenderType {
    Tile,
    Fireball,
    Shield,
    Swoop,
    Text,
}

pub enum Element {
    Tile(Tile),
    Fireball(Fireball),
    Shield(Shield),
    Swoop(Swoop),
    Text(Text),
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

impl From<Swoop> for Element {
    fn from(value: Swoop) -> Self {
        Self::Swoop(value)
    }
}

impl From<Text> for Element {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}

pub enum Renderer<'a> {
    Tile(InstancedShapeManager<'a>),
    Fireball(FireballManager<'a>),
    Shield(ShieldManager<'a>),
    Swoop(SwoopManager<'a>),
    Text(TextManager<'a>),
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

impl<'a> From<SwoopManager<'a>> for Renderer<'a> {
    fn from(value: SwoopManager<'a>) -> Self {
        Self::Swoop(value)
    }
}

impl<'a> From<TextManager<'a>> for Renderer<'a> {
    fn from(value: TextManager<'a>) -> Self {
        Self::Text(value)
    }
}

impl<'a> Renderer<'a> {
    fn render_type(&self) -> RenderType {
        match self {
            Renderer::Tile(_) => RenderType::Tile,
            Renderer::Fireball(_) => RenderType::Fireball,
            Renderer::Shield(_) => RenderType::Shield,
            Renderer::Swoop(_) => RenderType::Swoop,
            Renderer::Text(_) => RenderType::Text,
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
            Renderer::Swoop(swoop) => {
                if let Element::Swoop(s) = element {
                    swoop.push(s)
                }
            }
            Renderer::Text(text) => {
                if let Element::Text(t) = element {
                    text.push(t)
                }
            }
        }
    }

    pub fn draw(&mut self) {
        match self {
            Renderer::Tile(t) => t.draw(),
            Renderer::Fireball(f) => f.draw(),
            Renderer::Shield(s) => s.draw(),
            Renderer::Swoop(s) => s.draw(),
            Renderer::Text(t) => t.draw(),
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
            -1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, -1.0, 1.0, 0.0f32,
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
            Element::Swoop(swoop) => self
                .renderers
                .get_mut(&RenderType::Swoop)
                .map(|r| r.push(swoop)),
            Element::Text(text) => self
                .renderers
                .get_mut(&RenderType::Text)
                .map(|r| r.push(text)),
        };
    }

    pub fn draw(&mut self) {
        // render the scene first
        // for transparency to work properly
        // first render all opaque objects
        // then render translucents back to front
        self.framebuffer.with(|_| {
            FrameBuffer::clear();

            self.renderers.get_mut(&RenderType::Tile).map(|r| r.draw());

            self.renderers.get_mut(&RenderType::Text).map(|r| r.draw());
            self.renderers.get_mut(&RenderType::Swoop).map(|r| r.draw());
            self.renderers
                .get_mut(&RenderType::Fireball)
                .map(|r| r.draw());
            self.renderers
                .get_mut(&RenderType::Shield)
                .map(|r| r.draw());
        });

        // render the texture onto the monitor
        FrameBuffer::clear();
        self.vao.apply();
        self.shader.apply();
        self.start_time.elapsed().as_millis().uniform(0);
        // set crt brightness
        const CRT_LOADTIME: Duration = Duration::from_millis(1500);
        let p = self.start_time.elapsed().as_secs_f32() / CRT_LOADTIME.as_secs_f32();
        let brightness = ease::in_expo(p);
        brightness.uniform(1);

        if brightness >= 1.0 {
            // set void color
            let clear_color = Vec3::rgb(7, 14, 54).srgb_to_linear();
            gl::call!(ClearColor(clear_color.x, clear_color.y, clear_color.z, 1.0));
        }

        self.framebuffer.bind_texture(0);
        call!(DrawArrays(TRIANGLE_STRIP, 0, 4));
    }
}

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
