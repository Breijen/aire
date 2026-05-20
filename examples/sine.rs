use aire::{Engine, Source};
use std::f32::consts::TAU;
use std::thread;
use std::time::Duration;

struct Sine {
    frequency: f32,
    sample_rate: f32,
    phase: f32,
    duration_samples: usize,
    elapsed: usize,
}

impl Sine {
    fn new(frequency: f32, sample_rate: f32, duration_secs: f32) -> Self {
        Self {
            frequency,
            sample_rate,
            phase: 0.0,
            duration_samples: (sample_rate * duration_secs) as usize,
            elapsed: 0,
        }
    }
}

impl Source for Sine {
    fn next_sample(&mut self) -> f32 {
        let sample = (self.phase * TAU).sin() * 0.3;
        self.phase = (self.phase + self.frequency / self.sample_rate).fract();
        self.elapsed += 1;
        sample
    }

    fn is_finished(&self) -> bool {
        self.elapsed >= self.duration_samples
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;

    engine.add_source(Sine::new(440.0, engine.sample_rate() as f32, 2.0));

    thread::sleep(Duration::from_secs(3));

    Ok(())
}