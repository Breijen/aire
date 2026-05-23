use std::sync::atomic::{AtomicU64, Ordering};
use crossbeam_channel::{bounded, Sender};
use cpal::SampleRate;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::AireError;
use crate::handle::SoundHandle;
use crate::mixer::Mixer;
use crate::sound::Sound;

pub type SoundId = u64;

pub(crate) enum Command {
    AddSound(SoundId, Box<Sound>),
    Pause(SoundId),
    Resume(SoundId),
    Stop(SoundId),
    SetVolume(SoundId, f32),
    SetPan(SoundId, f32),
}

pub struct Engine {
    tx: Sender<Command>,
    next_id: AtomicU64,
    _stream: cpal::Stream,
    sample_rate: SampleRate,
}

impl Engine {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no output device available")?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate();
        let channels = config.channels() as usize;

        let (tx, rx) = bounded::<Command>(256);
        let mut mixer = Mixer::new(0.0);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _| {
                while let Ok(cmd) = rx.try_recv() {
                    mixer.apply(cmd);
                }
                for frame in data.chunks_mut(channels) {
                    let (left, right) = mixer.next_sample();
                    if let Some(l) = frame.get_mut(0) { *l = left; }
                    if let Some(r) = frame.get_mut(1) { *r = right; }
                }
            },
            |err| eprintln!("audio stream error: {}", err),
            None,
        )?;

        stream.play()?;

        Ok(Self {
            tx,
            next_id: AtomicU64::new(0),
            _stream: stream,
            sample_rate,
        })
    }

    pub fn add_sound(&self, sound: Sound) -> Result<SoundHandle, AireError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.tx.try_send(Command::AddSound(id, Box::new(sound)))
            .map_err(|_| AireError::CommandBufferFull)?;
        Ok(SoundHandle::new(id, self.tx.clone()))
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}
