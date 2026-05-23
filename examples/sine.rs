use aire::{Engine, Oscillator, Sound, Waveform};
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let engine = Engine::new()?;
    let rate = engine.sample_rate();

    let sine = Oscillator::new(Waveform::Sine, 440.0, rate)
        .amplitude(-6.0)
        .duration(1500);
    engine.add_sound(Sound::new(sine, -6.0, 0.5, rate))?;
    thread::sleep(Duration::from_millis(2000));

    let saw = Oscillator::new(Waveform::Saw, 220.0, rate)
        .amplitude(-22.0)
        .duration(1500);
    engine.add_sound(Sound::new(saw, -6.0, 0.5, rate))?;
    thread::sleep(Duration::from_millis(2000));

    let pulse = Oscillator::new(Waveform::Pulse, 330.0, rate)
        .amplitude(-22.0)
        .pulse_width(0.02)
        .duration(1500);
    engine.add_sound(Sound::new(pulse, -6.0, 0.5, rate))?;
    thread::sleep(Duration::from_millis(2000));

    Ok(())
}
