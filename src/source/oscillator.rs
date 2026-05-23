use std::f32::consts::TAU;
use crate::source::Source;
use crate::utils::{convert_db, milliseconds_to_samples};

/// The waveform shape produced by an [`Oscillator`].
pub enum Waveform {
    /// A smooth sine wave.
    Sine,
    /// A rising sawtooth wave.
    Saw,
    /// A falling sawtooth wave. Same harmonic content as [`Waveform::Saw`], mirrored.
    SawDown,
    /// A triangle wave.
    Triangle,
    /// A square wave with a fixed 50% duty cycle.
    Square,
    /// A pulse wave with a variable duty cycle. Set width via [`Oscillator::pulse_width`].
    /// At 0.5 it is identical to [`Waveform::Square`].
    Pulse,
}

/// A band-limited oscillator with six waveforms.
///
/// Configured at construction via a builder chain and handed to [`crate::Sound`] for playback.
/// Without a duration set, the oscillator runs indefinitely.
pub struct Oscillator {
    waveform: Waveform,
    frequency: f32,
    sample_rate: u32,
    phase: f32,
    amplitude: f32,
    duration_samples: Option<usize>,
    elapsed: usize,
    pulse_width: f32,
}

impl Oscillator {
    /// Creates a new oscillator. Runs indefinitely until a [`Oscillator::duration`] is set.
    /// Amplitude defaults to unity gain.
    pub fn new(waveform: Waveform, frequency: f32, device_rate: u32) -> Oscillator {
        Oscillator { waveform, frequency, sample_rate: device_rate, phase: 0.0, amplitude: 1.0, duration_samples: None, elapsed: 0, pulse_width: 0.5 }
    }

    /// Sets the oscillator amplitude in decibels. Defaults to `0.0`.
    pub fn amplitude(mut self, db: f32) -> Self {
        self.amplitude = convert_db(db);
        self
    }

    /// Sets the playback duration in milliseconds.
    pub fn duration(mut self, ms: u32) -> Self {
        self.duration_samples = Some(milliseconds_to_samples(ms, self.sample_rate) as usize);
        self
    }

    /// Sets the pulse width for [`Waveform::Pulse`]. Clamped to `0.01..0.99`.
    /// Has no effect on other waveforms.
    pub fn pulse_width(mut self, width: f32) -> Self {
        self.pulse_width = width.clamp(0.01, 0.99);
        self
    }
}

fn poly_blamp(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        dt * (0.5 * t * t - t + 0.5)
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        dt * (0.5 * t * t + t + 0.5)
    } else {
        0.0
    }
}

fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

impl Source for Oscillator {
    fn fill_buffer(&mut self, buffer: &mut [(f32, f32)]) {
        let dt = self.frequency / self.sample_rate as f32;

        for frame in buffer.iter_mut() {
            if self.duration_samples.map_or(false, |d| self.elapsed >= d) {
                *frame = (0.0, 0.0);
                continue;
            }

            let t = self.phase;
            let sample = match self.waveform {
                Waveform::Sine => (t * TAU).sin(),
                Waveform::Saw => {
                    let naive = 2.0 * t - 1.0;
                    naive - poly_blep(t, dt)
                }
                Waveform::SawDown => {
                    let naive = 1.0 - 2.0 * t;
                    naive + poly_blep(t, dt)
                }
                Waveform::Triangle => {
                    let t_shifted = (t + 0.75).fract();
                    let naive = 4.0 * (t_shifted - 0.5).abs() - 1.0;
                    // Slope changes by 8 at each corner,
                    // so the BLAMP correction is scaled by that magnitude
                    naive
                        - 8.0 * poly_blamp((t - 0.25).rem_euclid(1.0), dt)
                        + 8.0 * poly_blamp((t - 0.75).rem_euclid(1.0), dt)
                }
                Waveform::Square => {
                    let naive = if t < 0.5 { 1.0 } else { -1.0 };
                    naive + poly_blep(t, dt)
                        - poly_blep((t - 0.5).rem_euclid(1.0), dt)
                }
                Waveform::Pulse => {
                    let naive = if t < self.pulse_width { 1.0 } else { -1.0 };
                    naive + poly_blep(t, dt)
                        - poly_blep((t - self.pulse_width).rem_euclid(1.0), dt)
                }
            } * self.amplitude;

            self.phase = (self.phase + dt).fract();
            self.elapsed += 1;

            let fade = if let Some(d) = self.duration_samples {
                let fade_samples = ((self.sample_rate as f32 * 0.005) as usize).max(1);
                let remaining = d.saturating_sub(self.elapsed);
                (remaining as f32 / fade_samples as f32).min(1.0)
            } else {
                1.0
            };

            *frame = (sample * fade, sample * fade);
        }
    }

    fn is_finished(&self) -> bool {
        self.duration_samples.map_or(false, |d| self.elapsed >= d)
    }
}
