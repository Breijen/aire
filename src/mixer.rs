use crate::engine::{Command, SoundId};
use crate::source::Source;
use crate::sound::Sound;
use crate::utils;

pub(crate) struct Mixer {
    sources: Vec<(SoundId, Box<Sound>)>,
    volume: f32,
    scratch: Vec<(f32, f32)>,
}

impl Mixer {
    pub fn new(volume_db: f32) -> Self {
        Self {
            sources: Vec::new(),
            volume: utils::convert_db(volume_db),
            scratch: Vec::new(),
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

    pub fn fill_buffer(&mut self, buffer: &mut [(f32, f32)]) {
        self.sources.retain(|(_, s)| !s.is_finished());
        buffer.fill((0.0, 0.0));

        if self.scratch.len() < buffer.len() {
            self.scratch.resize(buffer.len(), (0.0, 0.0));
        }
        let scratch = &mut self.scratch[..buffer.len()];

        for (_, source) in &mut self.sources {
            scratch.fill((0.0, 0.0));
            source.fill_buffer(scratch);
            for (out, inp) in buffer.iter_mut().zip(scratch.iter()) {
                out.0 += inp.0;
                out.1 += inp.1;
            }
        }

        for frame in buffer.iter_mut() {
            frame.0 = (frame.0 * self.volume).clamp(-1.0, 1.0);
            frame.1 = (frame.1 * self.volume).clamp(-1.0, 1.0);
        }
    }
}
