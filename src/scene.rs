use std::mem;

use crate::{
    gl::{DrawContext, Shader},
    math::{Mat4, Vec3},
    render::{Instance, InstancedShapeManager},
};

#[derive(Default, Clone, Copy, PartialEq)]
pub enum Tile {
    #[default]
    Empty,
    Wall,
}

impl Tile {
    pub fn color(self) -> Vec3 {
        match self {
            Tile::Empty => Vec3::default(),
            Tile::Wall => (0.15).into(),
        }
    }
}

pub struct Scene<'a> {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,

    renderer: InstancedShapeManager<'a>,
}

impl<'a> Scene<'a> {
    pub fn new(ctx: &'a DrawContext, width: usize, height: usize) -> Self {
        let tiles = vec![Tile::default(); width * height];
        let mut renderer = InstancedShapeManager::quads(ctx, width * height);
        for y in 0..height {
            for x in 0..width {
                renderer
                    .new_instance(Some(Instance {
                        transform: Mat4::translate((x as f32, y as f32).into()),
                        col: Tile::default().color(),
                        ..Default::default()
                    }))
                    .expect("Failed to build scene, out VRAM for tiles");
            }
        }

        let mut this = Self {
            width,
            height,
            tiles,

            renderer,
        };

        // create walls
        let wall = Tile::Wall;

        for y in 0..height {
            for x in 0..width {
                let tile = if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                    wall
                } else {
                    Tile::default()
                };

                this.set_tile(x, y, tile);
            }
        }

        this
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Tile {
        self.tiles[x + y * self.width]
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        let id = x + y * self.width;
        let old_tile = mem::replace(&mut self.tiles[id], tile);
        if tile != old_tile {
            // send data to the gpu
            self.renderer.set_instance(
                id,
                Instance {
                    transform: Mat4::translate((x as f32, y as f32).into()),
                    col: tile.color(),
                    ..Default::default()
                },
            )
        }
    }

    pub fn draw(&self, shader: &Shader) {
        self.renderer.draw(shader)
    }
}
