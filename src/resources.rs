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
pub type Texture = Resource;
pub type Sound = Resource;
pub type Shader = &'static [Resource];

// TEXTURES //
pub mod textures {
    use super::Texture;

    pub const ICON: Texture = load!("textures/favicon.ico");

    pub mod text {
        use super::Texture;

        pub const SNEK: Texture = load!("textures/text/snek.png");
        pub const SNEK_GLITCH: Texture = load!("textures/text/snek-glitch.png");
        pub const CONTROLS: Texture = load!("textures/text/controls.png");
        pub const FRUIT: Texture = load!("textures/text/fruit.png");
        pub const FRUIT_GLITCH: Texture = load!("textures/text/fruit-glitch.png");
        pub const FRUIT_GLITCH_VARIANT: Texture = load!("textures/text/fruit-glitch-variant.png");
        pub const ATTACK: Texture = load!("textures/text/attack.png");
        pub const ATTACK_GLITCH: Texture = load!("textures/text/attack-glitch.png");
        pub const EMPOWER: Texture = load!("textures/text/empower.png");
        pub const EMPOWER_GLITCH: Texture = load!("textures/text/empower-glitch.png");
    }
}

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
    pub const SWOOP: Sound = load!("sounds/swoop.wav");
    pub const CRT_ON: Sound = load!("sounds/crt-on.wav");
    pub const CRT_CLICK: Sound = load!("sounds/crt-click.wav");
    pub const CRT_BUZZ: Sound = load!("sounds/crt-buzz.wav");
    pub const GLITCH_0: Sound = load!("sounds/glitch-0.wav");
    pub const GLITCH_1: Sound = load!("sounds/glitch-1.wav");
    pub const GLITCH_2: Sound = load!("sounds/glitch-2.wav");
    pub const GLITCH_3: Sound = load!("sounds/glitch-3.wav");
    pub const GLITCH_4: Sound = load!("sounds/glitch-4.wav");
    pub const GLITCH_5: Sound = load!("sounds/glitch-5.wav");
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

    pub const CRT: Shader = &[load!("shaders/crt.vert"), load!("shaders/crt.frag")];

    pub const SWOOP: Shader = &[load!("shaders/swoop.vert"), load!("shaders/swoop.frag")];

    pub const TEXT: Shader = &[load!("shaders/text.vert"), load!("shaders/text.frag")];
}
