use std::{
    collections::HashMap,
};

use self::{fireball::{Fireball, FireballManager}, instanced::{InstancedShapeManager, Tile}, shield::{Shield, ShieldManager}};


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
            },
            Renderer::Fireball(fire) => {
                if let Element::Fireball(f) = element {
                    fire.push(f)
                }
            },
            Renderer::Shield(shield) => {
                if let Element::Shield(s) = element {
                    shield.push(s)
                }
            },
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

#[derive(Default)]
pub struct RenderManager<'a> {
    renderers: HashMap<RenderType, Renderer<'a>>,
}

impl<'a> RenderManager<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_renderer(&mut self, renderer: impl Into<Renderer<'a>>) {
        let renderer = renderer.into();
        self.renderers.insert(renderer.render_type(), renderer);
    }

    pub fn push(&mut self, element: impl Into<Element>) {
        match element.into() {
            Element::Tile(tile) => self.renderers.get_mut(&RenderType::Tile).map(|r| r.push(tile)),
            Element::Fireball(fire) => self.renderers.get_mut(&RenderType::Fireball).map(|r| r.push(fire)),
            Element::Shield(shield) => self.renderers.get_mut(&RenderType::Shield).map(|r| r.push(shield)),
        };
    }

    pub fn draw(&mut self) {
        // for transparency to work properly
        // render back to front
        self.renderers.get_mut(&RenderType::Tile).map(|r| r.draw());
        self.renderers.get_mut(&RenderType::Shield).map(|r| r.draw());
        self.renderers.get_mut(&RenderType::Fireball).map(|r| r.draw());
    }
}
