use crate::utils;
use crate::source::Source;

pub struct Mixer {
    sources: Vec<Box<dyn Source>>,
    volume: f32
}

impl Mixer {
    pub fn new(volume: f32) -> Self {
        Self {
            sources: Vec::new(),
            volume: utils::convert_db(volume)
        }
    }

    pub fn add(&mut self, source: Box<dyn Source>) {
        self.sources.push(source);
    }

    pub fn next_sample(&mut self) -> f32 {
        self.sources.retain(|s| !s.is_finished());

        let mut sum = self.sources.iter_mut()
            .map(|s| s.next_sample())
            .sum::<f32>();

        sum *= self.volume;

        sum.clamp(-1.0, 1.0)
    }
}
