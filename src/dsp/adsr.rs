use crate::utils::{convert_db, milliseconds_to_samples};
use crate::dsp::effect::Effect;

/// Shape of the attack and release envelope segments.
pub enum Curve {
    /// Volume changes at a constant rate.
    Linear,
    /// Volume changes along a squared curve. Sounds more natural for
    /// percussive sources.
    Exponential,
}

/// An ADSR (Attack, Decay, Sustain, Release) amplitude envelope effect.
/// Shapes the volume of a sound over time. All time values are in milliseconds,
/// sustain amplitude is in decibels.
///
/// Attack- and Release-curves default to [`Curve::Linear`].
pub struct Adsr {
    attack: u32,
    decay: u32,
    sustain_amp: f32,
    sustain_time: u32,
    release: u32,
    current_sample: u32,
    stage: AdsrStage,
    attack_curve: Curve,
    release_curve: Curve,
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
            attack: milliseconds_to_samples(attack, device_rate).max(1),
            decay: milliseconds_to_samples(decay, device_rate).max(1),
            sustain_time: milliseconds_to_samples(sustain_time, device_rate),
            release: milliseconds_to_samples(release, device_rate).max(1),
            sustain_amp: convert_db(sustain_amp),
            current_sample: 0,
            stage: AdsrStage::Attack,
            attack_curve: Curve::Linear,
            release_curve: Curve::Linear,
        }
    }

    /// Sets the attack curve shape. Defaults to [`Curve::Linear`].
    pub fn attack_curve(mut self, curve: Curve) -> Self {
        self.attack_curve = curve;
        self
    }

    /// Sets the release curve shape. Defaults to [`Curve::Linear`].
    pub fn release_curve(mut self, curve: Curve) -> Self {
        self.release_curve = curve;
        self
    }
}

impl Adsr {
    fn process_sample(&mut self, (l, r): (f32, f32)) -> (f32, f32) {
        match self.stage {
            AdsrStage::Attack => {
                // increment first so the final sample reaches full peak
                self.current_sample += 1;
                let t = self.current_sample as f32 / self.attack as f32;
                let multiplier = match self.attack_curve {
                    Curve::Linear      => t,
                    Curve::Exponential => t * t,
                };
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
                // increment first so the final sample reaches full silence
                self.current_sample += 1;
                let t = self.current_sample as f32 / self.release as f32;
                let multiplier = match self.release_curve {
                    Curve::Linear      => self.sustain_amp * (1.0 - t),
                    Curve::Exponential => self.sustain_amp * (1.0 - t) * (1.0 - t),
                };
                if self.current_sample >= self.release {
                    self.stage = AdsrStage::Finished;
                }
                (l * multiplier, r * multiplier)
            }
            AdsrStage::Finished => (0.0, 0.0),
        }
    }
}

impl Effect for Adsr {
    fn process(&mut self, buffer: &mut [(f32, f32)]) {
        for frame in buffer.iter_mut() {
            *frame = self.process_sample(*frame);
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self.stage, AdsrStage::Finished)
    }
}
