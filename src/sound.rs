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

/// A playable sound with volume, pan, and an optional effect chain.
///
/// Wraps a [`Source`] and applies volume, stereo pan, and any attached
/// [`Effect`]s. Volume and pan changes are smoothed to avoid audio clicks.
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
    /// Creates a new sound.
    ///
    /// `volume_db` is the initial volume in decibels (0.0 is unity gain). `pan`
    /// is stereo position from `0.0` (full left) to `1.0` (full right), with
    /// `0.5` as center. `sample_rate` should match the device rate.
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

    /// Pauses playback.
    pub fn pause(&mut self) {
        if matches!(self.state, State::Playing) {
            self.pause_volume = self.volume_target;
            self.volume_target = 0.0;
            self.state = State::FadingToPause;
        }
    }

    /// Resumes playback.
    pub fn resume(&mut self) {
        match self.state {
            State::Paused | State::FadingToPause => {
                self.volume_target = self.pause_volume;
                self.state = State::Playing;
            }
            _ => {}
        }
    }

    /// Stops playback and removes the sound from the mixer.
    pub fn stop(&mut self) {
        if !matches!(self.state, State::Stopped | State::FadingToStop) {
            self.volume_target = 0.0;
            self.state = State::FadingToStop;
        }
    }

    /// Returns `true` if the sound is currently paused.
    pub fn is_paused(&self) -> bool {
        matches!(self.state, State::Paused | State::FadingToPause)
    }

    /// Adds an effect to the processing chain.
    pub fn add_effect(&mut self, effect: impl Effect + 'static) -> &mut Self {
        self.effects.push(Box::new(effect));
        self
    }

    /// Sets the volume in decibels.
    pub fn set_volume(&mut self, volume_db: f32) -> &mut Self {
        self.volume_target = convert_db(volume_db);
        self
    }

    /// Sets the pan from `0.0` (full left) to `1.0` (full right).
    pub fn set_pan(&mut self, pan: f32) -> &mut Self {
        self.pan_target = pan.clamp(0.0, 1.0);
        self
    }
}

impl Source for Sound {
    fn fill_buffer(&mut self, buffer: &mut [(f32, f32)]) {
        if matches!(self.state, State::Paused | State::Stopped) {
            buffer.fill((0.0, 0.0));
            return;
        }

        self.source.fill_buffer(buffer);

        for effect in &mut self.effects {
            effect.process(buffer);
        }

        let source_done = self.source.is_finished();
        let effects_done = !self.effects.is_empty() && self.effects.iter().any(|e| e.is_finished());
        if (source_done || effects_done) && matches!(self.state, State::Playing) {
            self.volume_target = 0.0;
            self.state = State::FadingToStop;
        }

        for frame in buffer.iter_mut() {
            self.volume += (self.volume_target - self.volume) * self.smooth_coeff;
            self.pan += (self.pan_target - self.pan) * self.smooth_coeff;

            if self.volume < SILENCE_THRESHOLD {
                match self.state {
                    State::FadingToPause => {
                        self.state = State::Paused;
                        self.volume = 0.0;
                    }
                    State::FadingToStop => {
                        self.state = State::Stopped;
                        self.volume = 0.0;
                    }
                    _ => {}
                }
            }

            if matches!(self.state, State::Paused | State::Stopped) {
                *frame = (0.0, 0.0);
            } else {
                let (l, r) = *frame;
                let pan_left = (self.pan * FRAC_PI_2).cos();
                let pan_right = (self.pan * FRAC_PI_2).sin();
                *frame = (l * self.volume * pan_left, r * self.volume * pan_right);
            }
        }
    }

    fn is_finished(&self) -> bool {
        matches!(self.state, State::Stopped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockSource { remaining: usize }

    impl MockSource {
        fn finite(n: usize) -> Self { Self { remaining: n } }
        fn endless() -> Self { Self { remaining: usize::MAX } }
    }

    impl Source for MockSource {
        fn fill_buffer(&mut self, buffer: &mut [(f32, f32)]) {
            for frame in buffer.iter_mut() {
                if self.remaining > 0 {
                    self.remaining = self.remaining.saturating_sub(1);
                    *frame = (1.0, 1.0);
                } else {
                    *frame = (0.0, 0.0);
                }
            }
        }
        fn is_finished(&self) -> bool { self.remaining == 0 }
    }

    fn tick(sound: &mut Sound, n: usize) {
        let mut buf = vec![(0.0f32, 0.0f32); n];
        sound.fill_buffer(&mut buf);
    }
    

    #[test]
    fn resume_mid_fade_restores_volume() {
        let mut s = Sound::new(MockSource::endless(), 0.0, 0.5, 44100);
        let vol = s.volume_target;
        s.pause();
        tick(&mut s, 100);
        s.resume();
        assert!(matches!(s.state, State::Playing));
        assert!((s.volume_target - vol).abs() < 0.001);
    }

    #[test]
    fn double_stop_does_not_panic() {
        let mut s = Sound::new(MockSource::endless(), 0.0, 0.5, 44100);
        s.stop();
        tick(&mut s, 5000);
        s.stop();
    }

    #[test]
    fn finished_when_source_exhausted() {
        let mut s = Sound::new(MockSource::finite(10), 0.0, 0.5, 44100);
        tick(&mut s, 10);    // source exhausts, triggers FadingToStop
        tick(&mut s, 5000);  // wait for fade to complete
        assert!(s.is_finished());
    }
}
