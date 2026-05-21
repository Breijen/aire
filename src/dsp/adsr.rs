use crate::Source;
use crate::utils::{convert_db, milliseconds_to_samples};

pub struct Adsr<S: Source>{
    inner: S,
    attack: u32,
    decay: u32,
    sustain_amp: f32,
    sustain_time: u32,
    release: u32,
    current_sample: u32,
    stage: AdsrStage,
}

enum AdsrStage {
    Attack,
    Decay,
    Sustain,
    Release,
    Finished,
}

impl<S: Source> Adsr<S> {
    pub fn new(inner: S, device_rate: u32, mut attack: u32, mut decay: u32, mut sustain_time: u32, mut release: u32, mut sustain_amp: f32) -> Self {
        attack = milliseconds_to_samples(attack, device_rate);
        decay = milliseconds_to_samples(decay, device_rate);
        sustain_time = milliseconds_to_samples(sustain_time, device_rate);
        release = milliseconds_to_samples(release, device_rate);

        sustain_amp = convert_db(sustain_amp);

        Adsr {
            inner,
            attack,
            decay,
            sustain_time,
            sustain_amp,
            release,
            current_sample: 0,
            stage: AdsrStage::Attack,
        }
    }
}

impl<S: Source> Source for Adsr<S> {
    fn next_sample(&mut self) -> f32 {
        match self.stage {
            AdsrStage::Attack => {
                let multiplier = self.current_sample as f32 / self.attack as f32;
                let sample = self.inner.next_sample() * multiplier;
                self.current_sample += 1;
                if self.current_sample >= self.attack {
                    self.stage = AdsrStage::Decay;
                    self.current_sample = 0;
                }
                sample
            }
            AdsrStage::Decay => {
                let t = self.current_sample as f32 / self.decay as f32;
                let multiplier = 1.0 - (1.0 - self.sustain_amp) * t;
                let sample = self.inner.next_sample() * multiplier;
                self.current_sample += 1;
                if self.current_sample >= self.decay {
                    self.stage = AdsrStage::Sustain;
                    self.current_sample = 0;
                }
                sample
            }
            AdsrStage::Sustain => {
                let multiplier = self.sustain_amp;
                let sample = self.inner.next_sample() * multiplier;
                self.current_sample += 1;
                if self.current_sample >= self.sustain_time {
                    self.stage = AdsrStage::Release;
                    self.current_sample = 0;
                }
                sample
            }
            AdsrStage::Release => {
                let t = self.current_sample as f32 / self.release as f32;
                let multiplier = self.sustain_amp * (1.0 - t);
                let sample = self.inner.next_sample() * multiplier;
                self.current_sample += 1;
                if self.current_sample >= self.release {
                    self.stage = AdsrStage::Finished;
                }
                sample
            }
            AdsrStage::Finished => {
                0.0
            }
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self.stage, AdsrStage::Finished) || self.inner.is_finished()
    }
}