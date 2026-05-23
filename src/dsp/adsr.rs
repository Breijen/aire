use crate::utils::{convert_db, milliseconds_to_samples};
use crate::dsp::effect::Effect;

/// An ADSR (Attack, Decay, Sustain, Release) amplitude envelope effect.
/// Shapes the volume of a sound over time. All time values are in milliseconds,
/// sustain amplitude is in decibels.
pub struct Adsr {
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

impl Adsr {
    /// Creates a new ADSR envelope.
    ///
    /// - `device_rate`: sample rate from the engine
    /// - `attack`: rise time in milliseconds
    /// - `decay`: fall time from peak to sustain level in milliseconds
    /// - `sustain_time`: time held at sustain level in milliseconds
    /// - `release`: fade-out time in milliseconds
    /// - `sustain_amp`: sustain amplitude in decibels
    pub fn new(device_rate: u32, attack: u32, decay: u32, sustain_time: u32, release: u32, sustain_amp: f32) -> Self {
        Adsr {
            attack: milliseconds_to_samples(attack, device_rate),
            decay: milliseconds_to_samples(decay, device_rate),
            sustain_time: milliseconds_to_samples(sustain_time, device_rate),
            release: milliseconds_to_samples(release, device_rate),
            sustain_amp: convert_db(sustain_amp),
            current_sample: 0,
            stage: AdsrStage::Attack,
        }
    }
}

impl Effect for Adsr {
    fn process(&mut self, (l, r): (f32, f32)) -> (f32, f32) {
        match self.stage {
            AdsrStage::Attack => {
                let multiplier = self.current_sample as f32 / self.attack as f32;
                self.current_sample += 1;
                if self.current_sample >= self.attack {
                    self.stage = AdsrStage::Decay;
                    self.current_sample = 0;
                }
                (l * multiplier, r * multiplier)
            }
            AdsrStage::Decay => {
                let t = self.current_sample as f32 / self.decay as f32;
                let multiplier = 1.0 - (1.0 - self.sustain_amp) * t;
                self.current_sample += 1;
                if self.current_sample >= self.decay {
                    self.stage = AdsrStage::Sustain;
                    self.current_sample = 0;
                }
                (l * multiplier, r * multiplier)
            }
            AdsrStage::Sustain => {
                self.current_sample += 1;
                if self.current_sample >= self.sustain_time {
                    self.stage = AdsrStage::Release;
                    self.current_sample = 0;
                }
                (l * self.sustain_amp, r * self.sustain_amp)
            }
            AdsrStage::Release => {
                let t = self.current_sample as f32 / self.release as f32;
                let multiplier = self.sustain_amp * (1.0 - t);
                self.current_sample += 1;
                if self.current_sample >= self.release {
                    self.stage = AdsrStage::Finished;
                }
                (l * multiplier, r * multiplier)
            }
            AdsrStage::Finished => (0.0, 0.0),
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self.stage, AdsrStage::Finished)
    }
}