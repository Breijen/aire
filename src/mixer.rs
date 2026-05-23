use crate::engine::{Command, SoundId};
use crate::source::Source;
use crate::sound::Sound;
use crate::utils;

pub(crate) struct Mixer {
    sources: Vec<(SoundId, Box<Sound>)>,
    volume: f32,
}

impl Mixer {
    pub fn new(volume_db: f32) -> Self {
        Self {
            sources: Vec::new(),
            volume: utils::convert_db(volume_db),
        }
    }

    pub fn apply(&mut self, command: Command) {
        match command {
            Command::AddSound(id, sound) => self.sources.push((id, sound)),
            Command::Pause(id) => {
                if let Some((_, s)) = self.sources.iter_mut().find(|(sid, _)| *sid == id) {
                    s.pause();
                }
            }
            Command::Resume(id) => {
                if let Some((_, s)) = self.sources.iter_mut().find(|(sid, _)| *sid == id) {
                    s.resume();
                }
            }
            Command::Stop(id) => {
                if let Some((_, s)) = self.sources.iter_mut().find(|(sid, _)| *sid == id) {
                    s.stop();
                }
            }
            Command::SetVolume(id, db) => {
                if let Some((_, s)) = self.sources.iter_mut().find(|(sid, _)| *sid == id) {
                    s.set_volume(db);
                }
            }
            Command::SetPan(id, pan) => {
                if let Some((_, s)) = self.sources.iter_mut().find(|(sid, _)| *sid == id) {
                    s.set_pan(pan);
                }
            }
        }
    }

    pub fn next_sample(&mut self) -> (f32, f32) {
        self.sources.retain(|(_, s)| !s.is_finished());

        let (left, right) = self.sources.iter_mut()
            .map(|(_, s)| s.next_sample())
            .fold((0.0f32, 0.0f32), |(l, r), (sl, sr)| (l + sl, r + sr));

        (
            (left * self.volume).clamp(-1.0, 1.0),
            (right * self.volume).clamp(-1.0, 1.0),
        )
    }
}
