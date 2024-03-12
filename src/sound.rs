use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use rand::{thread_rng, Rng};
use soloud::{AudioExt, LoadExt, Soloud, Wav};

use crate::{
    common::{Error, Result},
    resources::Resource,
};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Sounds {
    Die,
    Eat,
    ShieldUp,
    Fireball,
    Move,
    RoomUnlocked,
    CameraPan,
    Swoop,
    CrtOn,
    CrtClick,
    CrtBuzz,
    Glitch0,
    Glitch1,
    Glitch2,
    Glitch3,
    Glitch4,
    Glitch5,

    _NumSounds,
}

impl Sounds {
    fn resource(self) -> Resource {
        use crate::resources::sounds::*;
        match self {
            Self::Die => DIE,
            Self::Eat => EAT,
            Self::ShieldUp => SHIELD_UP,
            Self::Fireball => FIREBALL,
            Self::Move => MOVE,
            Self::RoomUnlocked => ROOM_UNLOCKED,
            Self::CameraPan => CAMERA_PAN,
            Self::Swoop => SWOOP,
            Self::CrtOn => CRT_ON,
            Self::CrtClick => CRT_CLICK,
            Self::CrtBuzz => CRT_BUZZ,
            Self::Glitch0 => GLITCH_0,
            Self::Glitch1 => GLITCH_1,
            Self::Glitch2 => GLITCH_2,
            Self::Glitch3 => GLITCH_3,
            Self::Glitch4 => GLITCH_4,
            Self::Glitch5 => GLITCH_5,

            Self::_NumSounds => panic!(),
        }
    }

    pub fn glitch() -> Self {
        let mut rng = thread_rng();
        let first_glitch = Self::Glitch0;
        let last_glitch = Self::Glitch5;
        let glitch = rng.gen_range(first_glitch as u8 ..= last_glitch as u8);
        Self::try_from(glitch).unwrap()
    }
}

impl TryFrom<u8> for Sounds {
    type Error = Error;

    fn try_from(value: u8) -> Result<Sounds> {
        use Sounds as S;
        Ok(match value {
            0 => S::Die,
            1 => S::Eat,
            2 => S::ShieldUp,
            3 => S::Fireball,
            4 => S::Move,
            5 => S::RoomUnlocked,
            6 => S::CameraPan,
            7 => S::Swoop,
            8 => S::CrtOn,
            9 => S::CrtClick,
            10 => S::CrtBuzz,
            11 => S::Glitch0,
            12 => S::Glitch1,
            13 => S::Glitch2,
            14 => S::Glitch3,
            15 => S::Glitch4,
            16 => S::Glitch5,

            _ => Err(Error::InvalidSoundId)?,
        })
    }
}

pub struct SoundManager {
    tx: Sender<Sounds>,
}

impl SoundManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        let _handle = Self::start_engine(rx);
        Self { tx }
    }

    fn start_engine(sound_queue: Receiver<Sounds>) {
        // run the engine
        thread::spawn(move || {
            let sl = Soloud::default().unwrap();
            // load sounds
            let mut sounds = Vec::with_capacity(Sounds::_NumSounds as _);
            for sound_id in 0..(Sounds::_NumSounds as u8) {
                // don't forget to add new sounds to the conversion table in try_from
                let sound = Sounds::try_from(sound_id).unwrap();
                let mut wav = Wav::default();
                wav.load_mem(sound.resource())
                    .expect("can't find sound file");
                sounds.push(wav);
            }

            loop {
                if let Ok(sound) = sound_queue.recv() {
                    sl.play(&sounds[sound as usize]);
                    sl.voice_count();
                } else {
                    return;
                }
            }
        });
    }

    pub fn play(&self, sound: Sounds) {
        let _ = self.tx.send(sound);
    }

    pub fn player(&self) -> Player {
        Player {
            tx: self.tx.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Player {
    tx: Sender<Sounds>,
}

impl Player {
    pub fn play(&self, sound: Sounds) {
        let _ = self.tx.send(sound);
    }
}
