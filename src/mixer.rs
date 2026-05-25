use std::collections::HashMap;

use crate::engine::{Command, SoundId};
use crate::sound::Sound;
use crate::source::Source;
use crate::utils;

struct GroupState {
    volume: f32,
    pan: f32,
}

impl Default for GroupState {
    fn default() -> Self {
        Self { volume: 1.0, pan: 0.5 }
    }
}

pub(crate) struct Mixer {
    sources: Vec<(SoundId, Box<Sound>)>,
    sound_groups: HashMap<SoundId, Vec<String>>,
    groups: HashMap<String, GroupState>,
    master_volume: f32,
    scratch: Vec<(f32, f32)>,
}

impl Mixer {
    pub fn new(volume_db: f32) -> Self {
        Self {
            sources: Vec::new(),
            sound_groups: HashMap::new(),
            groups: HashMap::new(),
            master_volume: utils::convert_db(volume_db),
            scratch: Vec::new(),
        }
    }

    fn find_source(&mut self, id: SoundId) -> Option<&mut Sound> {
        self.sources.iter_mut()
            .find(|(sid, _)| *sid == id)
            .map(|(_, s)| s.as_mut())
    }

    pub fn apply(&mut self, command: Command) {
        match command {
            Command::AddSound(id, sound) => self.sources.push((id, sound)),
            Command::Pause(id)           => { if let Some(s) = self.find_source(id) { s.pause(); } }
            Command::Resume(id)          => { if let Some(s) = self.find_source(id) { s.resume(); } }
            Command::Stop(id)            => { if let Some(s) = self.find_source(id) { s.stop(); } }
            Command::SetVolume(id, db)   => { if let Some(s) = self.find_source(id) { s.set_volume(db); } }
            Command::SetPan(id, pan)     => { if let Some(s) = self.find_source(id) { s.set_pan(pan); } }
            Command::SetMasterVolume(db) => {
                self.master_volume = utils::convert_db(db);
            }
            Command::SetGroupVolume(name, db) => {
                self.groups.entry(name).or_default().volume = utils::convert_db(db);
            }
            Command::SetGroupPan(name, pan) => {
                // pan is already clamped at the GroupHandle entry point
                self.groups.entry(name).or_default().pan = pan;
            }
            Command::AddToGroup(id, name) => {
                self.groups.entry(name.clone()).or_default();
                self.sound_groups.entry(id).or_insert_with(Vec::new).push(name);
            }
        }
    }

    pub fn fill_buffer(&mut self, buffer: &mut [(f32, f32)]) {
        let mut finished_ids = Vec::new();
        self.sources.retain(|(id, s)| {
            if s.is_finished() { finished_ids.push(*id); false } else { true }
        });
        for id in finished_ids {
            self.sound_groups.remove(&id);
        }

        buffer.fill((0.0, 0.0));

        if self.scratch.len() < buffer.len() {
            self.scratch.resize(buffer.len(), (0.0, 0.0));
        }
        let scratch = &mut self.scratch[..buffer.len()];

        for (id, source) in &mut self.sources {
            scratch.fill((0.0, 0.0));
            source.fill_buffer(scratch);

            if let Some(names) = self.sound_groups.get(id) {
                // Fold all group gains into one multiplier before touching the buffer,
                // so we make a single pass regardless of how many groups the sound is in.
                let mut gl = 1.0f32;
                let mut gr = 1.0f32;
                for name in names {
                    if let Some(group) = self.groups.get(name) {
                        gl *= (2.0 * (1.0 - group.pan)).min(1.0) * group.volume;
                        gr *= (2.0 * group.pan).min(1.0) * group.volume;
                    }
                }
                if gl != 1.0 || gr != 1.0 {
                    for frame in scratch.iter_mut() {
                        frame.0 *= gl;
                        frame.1 *= gr;
                    }
                }
            }

            for (out, inp) in buffer.iter_mut().zip(scratch.iter()) {
                out.0 += inp.0;
                out.1 += inp.1;
            }
        }

        for frame in buffer.iter_mut() {
            frame.0 = (frame.0 * self.master_volume).clamp(-1.0, 1.0);
            frame.1 = (frame.1 * self.master_volume).clamp(-1.0, 1.0);
        }
    }
}
