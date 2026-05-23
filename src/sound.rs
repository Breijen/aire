use std::f32::consts::{FRAC_PI_2, TAU};
use crate::Source;
use crate::utils::convert_db;
use crate::dsp::effect::Effect;

const SILENCE_THRESHOLD: f32 = 0.0001;

enum State {
    Playing,
    FadingToPause,
    Paused,
    FadingToStop,
    Stopped,
}

pub struct Sound {
    source: Box<dyn Source>,
    volume: f32,
    volume_target: f32,
    pause_volume: f32,
    pan: f32,
    pan_target: f32,
    smooth_coeff: f32,
    effects: Vec<Box<dyn Effect>>,
    state: State,
}

impl Sound {
    pub fn new(source: impl Source + 'static, volume_db: f32, pan: f32, sample_rate: u32) -> Self {
        let volume = convert_db(volume_db);
        let pan = pan.clamp(0.0, 1.0);
        let smooth_coeff = 1.0 - (-TAU * 30.0 / sample_rate as f32).exp();
        Self {
            source: Box::new(source),
            volume,
            volume_target: volume,
            pause_volume: volume,
            pan,
            pan_target: pan,
            smooth_coeff,
            effects: Vec::new(),
            state: State::Playing,
        }
    }

    pub fn pause(&mut self) {
        if matches!(self.state, State::Playing) {
            self.pause_volume = self.volume_target;
            self.volume_target = 0.0;
            self.state = State::FadingToPause;
        }
    }

    pub fn resume(&mut self) {
        match self.state {
            State::Paused | State::FadingToPause => {
                self.volume_target = self.pause_volume;
                self.state = State::Playing;
            }
            _ => {}
        }
    }

    pub fn stop(&mut self) {
        if !matches!(self.state, State::Stopped | State::FadingToStop) {
            self.volume_target = 0.0;
            self.state = State::FadingToStop;
        }
    }

    pub fn is_paused(&self) -> bool {
        matches!(self.state, State::Paused | State::FadingToPause)
    }

    pub fn add_effect(&mut self, effect: impl Effect + 'static) -> &mut Self {
        self.effects.push(Box::new(effect));
        self
    }

    pub fn set_volume(&mut self, volume_db: f32) -> &mut Self {
        self.volume_target = convert_db(volume_db);
        self
    }

    pub fn set_pan(&mut self, pan: f32) -> &mut Self {
        self.pan_target = pan.clamp(0.0, 1.0);
        self
    }
}

impl Source for Sound {
    fn next_sample(&mut self) -> (f32, f32) {
        if matches!(self.state, State::Paused | State::Stopped) {
            return (0.0, 0.0);
        }

        self.volume += (self.volume_target - self.volume) * self.smooth_coeff;
        self.pan += (self.pan_target - self.pan) * self.smooth_coeff;

        if self.volume < SILENCE_THRESHOLD {
            match self.state {
                State::FadingToPause => {
                    self.state = State::Paused;
                    self.volume = 0.0;
                    return (0.0, 0.0);
                }
                State::FadingToStop => {
                    self.state = State::Stopped;
                    self.volume = 0.0;
                    return (0.0, 0.0);
                }
                _ => {}
            }
        }

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
        matches!(self.state, State::Stopped)
            || self.source.is_finished()
            || self.effects.iter().any(|e| e.is_finished())
    }
}
