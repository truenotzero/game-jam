use crate::math::Vec3;


#[derive(Default, Clone, Copy)]
pub enum Tile {
    #[default]
    Empty,
    Wall,
}

impl Tile {
    pub fn color(self) -> Vec3 {
        match self {
            Tile::Empty => Vec3::default(),
            Tile::Wall => 0.3.into(),
        }
    }
}

pub struct Scene {
    width: usize,
    height: usize,
    tiles: Vec<Tile>,
}

impl Scene {
    pub fn new(width: usize, height: usize) -> Self {
        let tiles = vec![Tile::default(); width * height];
        let mut this = Self {
            width,
            height,
            tiles,
        };

        // create walls
        let wall = Tile::Wall;
        for x in 0..width {
            this.set_tile(x, 0, wall);
            this.set_tile(x, height - 1, wall);
        }

        for y in 0..height {
            this.set_tile(0, y, wall);
            this.set_tile(width - 1, y, wall);
        }

        this
    }

    pub fn get_tile(&self, x: usize, y: usize) -> Tile {
        self.tiles[x + y * self.width]
    }

    pub fn set_tile(&mut self, x: usize, y: usize, tile: Tile) {
        self.tiles[x + y * self.width] = tile;
    }
}
