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

    pub fn srgb_to_linear(self) -> Self {
        Self {
            snake: self.snake.srgb_to_linear(),
            wall: self.wall.srgb_to_linear(),
            background: self.background.srgb_to_linear(),
            fruit: self.fruit.srgb_to_linear(),
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
    .srgb_to_linear()
}

pub fn bright_pastel() -> Palette {
    let green = Vec3::hexcode("5EFC8D").unwrap();
    let lavender = Vec3::hexcode("FDECEF").unwrap();
    let pink = Vec3::hexcode("EF476F").unwrap();
    let indigo = Vec3::hexcode("8377D1").unwrap();
    let sunglow = Vec3::hexcode("FFD166").unwrap();

    Palette {
        snake: green,
        wall: indigo,
        background: lavender,
        fruit: sunglow,
        // enemy: pink
    }
    .srgb_to_linear()
}

pub fn dark_pastel() -> Palette {
    let dark_blue = Vec3::hexcode("102542").unwrap();
    let light_blue = Vec3::hexcode("9DCDC0").unwrap();
    let pink = Vec3::hexcode("EF476F").unwrap();
    let indigo = Vec3::hexcode("8377D1").unwrap();
    let sunglow = Vec3::hexcode("FFD166").unwrap();

    Palette {
        snake: light_blue,
        wall: indigo,
        background: dark_blue,
        fruit: sunglow,
        // enemy: pink
    }
    .srgb_to_linear()
}
