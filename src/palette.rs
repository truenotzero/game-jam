use crate::math::Vec3;

#[derive(Default, Clone, Copy)]
pub enum PaletteKey {
    #[default]
    None,
    Snake,
    Wall,
    Background,
    Fruit,
}

#[derive(Clone, Copy)]
pub struct Palette {
    pub snake: Vec3,
    pub wall: Vec3,
    pub background: Vec3,
    pub fruit: Vec3,
}

impl Palette {
    pub fn get(self, key: PaletteKey) -> Vec3 {
        match key {
            PaletteKey::None => Vec3::default(),
            PaletteKey::Snake => self.snake,
            PaletteKey::Wall => self.wall,
            PaletteKey::Background => self.background,
            PaletteKey::Fruit => self.fruit,
        }
    }
}

pub fn aperture() -> Palette {
    let offwhite = Vec3::rgb(0xEA, 0xDF, 0xB4);
    let light_blue = Vec3::rgb(0x9B, 0xB0, 0xC1);
    let dark_blue = Vec3::rgb(0x51, 0x82, 0x9B);
    let orange = Vec3::rgb(0xF6, 0x99, 0x5C);
    Palette {
        snake: offwhite,
        wall: light_blue,
        background: dark_blue,
        fruit: orange,
    }
}
