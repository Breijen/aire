use std::sync::atomic::{AtomicU64, Ordering};
use crossbeam_channel::{bounded, Sender};
use cpal::SampleRate;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::error::AireError;
use crate::handle::{GroupHandle, SoundHandle};
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
    SetMasterVolume(f32),
    SetGroupVolume(String, f32),
    SetGroupPan(String, f32),
    AddToGroup(SoundId, String),
}

/// Configuration for the audio engine.
pub struct EngineConfig {
    /// Size of the lock-free command queue between the main thread and the
    /// audio thread. Each call to `add_sound`, `set_volume`, etc. consumes
    /// one slot. Returns [`AireError::CommandBufferFull`] when the queue is
    /// full. Increase this if your game sends many commands in a single frame.
    /// Default: `256`.
    pub command_buffer_size: usize,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self { command_buffer_size: 256 }
    }
}

/// The audio engine. Opens the default output device and drives the mixer
/// on a real-time audio thread. All communication with the audio thread
/// is lock-free and safe to call from any thread.
pub struct Engine {
    tx: Sender<Command>,
    next_id: AtomicU64,
    _stream: cpal::Stream,
    sample_rate: SampleRate,
}

impl Engine {
    /// Opens the default output device and starts the audio stream with default settings.
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        Self::with_config(EngineConfig::default())
    }

    /// Opens the default output device with custom [`EngineConfig`] settings.
    pub fn with_config(config: EngineConfig) -> Result<Self, Box<dyn std::error::Error>> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or("no output device available")?;

        let device_config = device.default_output_config()?;
        let sample_rate = device_config.sample_rate();
        let channels = device_config.channels() as usize;

        let (tx, rx) = bounded::<Command>(config.command_buffer_size);
        let mut mixer = Mixer::new(0.0);
        let mut mixer_buffer: Vec<(f32, f32)> = Vec::new();

        let stream = device.build_output_stream(
            &device_config.into(),
            move |data: &mut [f32], _| {
                while let Ok(cmd) = rx.try_recv() {
                    mixer.apply(cmd);
                }
                let frames = data.len() / channels;
                if mixer_buffer.len() < frames {
                    mixer_buffer.resize(frames, (0.0, 0.0));
                }
                let buf = &mut mixer_buffer[..frames];
                mixer.fill_buffer(buf);
                for (frame, &(left, right)) in data.chunks_mut(channels).zip(buf.iter()) {
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

    /// Submits a sound for playback and returns a handle for controlling it.
    /// Returns [`AireError::CommandBufferFull`] if the command queue is full.
    pub fn add_sound(&self, sound: Sound) -> Result<SoundHandle, AireError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.tx.try_send(Command::AddSound(id, Box::new(sound)))
            .map_err(|_| AireError::CommandBufferFull)?;
        Ok(SoundHandle::new(id, self.tx.clone()))
    }

    /// Returns the device sample rate in Hz.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Sets the master volume in decibels. Applied after all group and per-sound volumes.
    pub fn set_master_volume(&self, db: f32) -> Result<(), AireError> {
        self.tx.try_send(Command::SetMasterVolume(db))
            .map_err(|_| AireError::CommandBufferFull)
    }

    /// Returns a [`GroupHandle`] for the named group, creating it if it does not exist.
    ///
    /// Use groups to control volume and pan for a category of sounds (e.g. `"music"`, `"sfx"`).
    /// Add sounds to the group with [`GroupHandle::add`].
    pub fn group(&self, name: &str) -> GroupHandle {
        GroupHandle::new(name.to_string(), self.tx.clone())
    }
}
