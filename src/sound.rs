use std::f32::consts::FRAC_PI_2;
use crate::Source;
use crate::utils::convert_db;
use crate::dsp::effect::Effect;

pub struct Sound {
    source: Box<dyn Source>,
    volume: f32,
    pan: f32,
    effects: Vec<Box<dyn Effect>>,
}

impl Sound {
    pub fn new(source: impl Source + 'static, volume_db: f32, pan: f32) -> Self {
        Self {
            source: Box::new(source),
            volume: convert_db(volume_db),
            pan: pan.clamp(0.0, 1.0),
            effects: Vec::new(),
        }
    }

    pub fn add_effect(&mut self, effect: impl Effect + 'static) -> &mut Self {
        self.effects.push(Box::new(effect));
        self
    }

    pub fn set_volume(&mut self, volume_db: f32) -> &mut Self {
        self.volume = convert_db(volume_db);
        self
    }

    pub fn set_pan(&mut self, pan: f32) -> &mut Self {
        self.pan = pan.clamp(0.0, 1.0);
        self
    }
}

impl Source for Sound {
    fn next_sample(&mut self) -> (f32, f32) {
        let mut sample = self.source.next_sample();

        for effect in &mut self.effects {
            sample = effect.process(sample);
        }

        let (l, r) = sample;
        let pan_left = (self.pan * FRAC_PI_2).cos();
        let pan_right = (self.pan * FRAC_PI_2).sin();
        (l * self.volume * pan_left, r * self.volume * pan_right)
    }

    fn is_finished(&self) -> bool {
        self.source.is_finished() || self.effects.iter().any(|e| e.is_finished())
    }
}
