use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};
use cpal::{SampleRate};
use crate::mixer::Mixer;
use crate::source::Source;

pub struct Engine {
    mixer: Arc<Mutex<Mixer>>,
    _stream: cpal::Stream,
    sample_rate: SampleRate
}

impl Engine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no output device available")?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate();

        let mixer = Arc::new(Mutex::new(Mixer::new(0.0)));
        let mixer_clone = Arc::clone(&mixer);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                let mut mixer = mixer_clone.lock().unwrap();
                for sample in data.iter_mut() {
                    *sample = mixer.next_sample();
                }
            },
            |err| eprintln!("audio stream error: {}", err),
            None,
        )?;

        stream.play()?;

        Ok(Self {
            mixer,
            _stream: stream,
            sample_rate
        })
    }

    pub fn add_source(&self, source: impl Source + 'static) {
        self.mixer.lock().unwrap().add(Box::new(source));
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}