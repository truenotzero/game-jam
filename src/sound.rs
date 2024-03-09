use std::{
    collections::HashMap,
    path::Path,
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use soloud::{AudioExt, LoadExt, Soloud, Wav};

use crate::common::{Error, Result};

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Sounds {
    Die,
    Eat,
    ShieldUp,

    _NumSounds,
}

impl Sounds {
    fn path(self) -> &'static Path {
        let path_string = match self {
            Sounds::Die => "res/sounds/die.wav",
            Sounds::Eat => "res/sounds/eat.wav",
            Sounds::ShieldUp => "res/sounds/shield-up.wav",
            Sounds::_NumSounds => panic!(),
        };

        path_string.as_ref()
    }
}

impl TryFrom<u8> for Sounds {
    type Error = Error;

    fn try_from(value: u8) -> Result<Sounds> {
        Ok(match value {
            0 => Sounds::Die,
            1 => Sounds::Eat,
            2 => Sounds::ShieldUp,
            _ => Err(Error::InvalidSoundId)?,
        })
    }
}

#[derive(Clone)]
pub struct SoundManager {
    tx: Sender<Sounds>,
}

impl SoundManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();

        Self::start_engine(rx);
        Self { tx }
    }

    fn start_engine(sound_queue: Receiver<Sounds>) {
        // load sounds
        let mut sounds = Vec::with_capacity(Sounds::_NumSounds as _);
        for sound_id in 0..(Sounds::_NumSounds as u8) {
            // don't forget to add new sounds to the conversion table in try_from
            let sound = Sounds::try_from(sound_id).unwrap();
            let mut wav = Wav::default();
            wav.load(sound.path()).expect("can't find sound file");
            sounds.push(wav);
        }

        let sl = Soloud::default().unwrap();
        // run the engine
        thread::spawn(move || loop {
            if let Ok(sound) = sound_queue.recv() {
                sl.play(&sounds[sound as usize]);
                sl.voice_count();
            } else {
                return;
            }
        });
    }

    pub fn play(&self, sound: Sounds) {
        let _ = self.tx.send(sound);
    }
}
