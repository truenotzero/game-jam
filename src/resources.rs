macro_rules! load {
    ($path:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/res/", $path))
    };
}

// macro_rules! res {
//     ($name:ident = $path:literal) => {
//         pub const $name: &'static [u8] = load!($path);
//     };
// }

pub type Resource = &'static [u8];
pub type Sound = Resource;
pub type Shader = &'static [Resource];

// WINDOW ICON //
pub const ICON: Resource = load!("favicon.ico");

// SOUNDS //
pub mod sounds {
    use super::Sound;

    pub const DIE: Sound = load!("sounds/die.wav");
    pub const EAT: Sound = load!("sounds/eat.wav");
    pub const SHIELD_UP: Sound = load!("sounds/shield-up.wav");
    pub const FIREBALL: Sound = load!("sounds/fireball.wav");
    pub const MOVE: Sound = load!("sounds/move.wav");
    pub const ROOM_UNLOCKED: Sound = load!("sounds/room-unlocked.wav");
    pub const CAMERA_PAN: Sound = load!("sounds/camera-pan.wav");
}

// SHADERS //
pub mod shaders {
    use super::Shader;

    pub const INSTANCED: Shader = &[
        load!("shaders/instanced.vert"),
        load!("shaders/instanced.frag"),
    ];

    pub const FIREBALL: Shader = &[
        load!("shaders/fireball.vert"),
        load!("shaders/fireball.frag"),
        load!("shaders/fireball.geom"),
    ];

    pub const SHIELD: Shader = &[
        load!("shaders/shield.vert"),
        load!("shaders/shield.frag"),
        load!("shaders/shield.geom"),
    ];
}
