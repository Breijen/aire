use crossbeam_channel::Sender;
use crate::error::AireError;
use crate::engine::{Command, SoundId};

/// A handle for controlling a sound after it has been added to the engine.
/// Obtained from [`crate::Engine::add_sound`]. All methods return [`AireError::CommandBufferFull`]
/// if the command queue is full.
pub struct SoundHandle {
    id: SoundId,
    tx: Sender<Command>,
}

impl SoundHandle {
    pub(crate) fn new(id: SoundId, tx: Sender<Command>) -> Self {
        Self { id, tx }
    }

    /// Pauses playback.
    pub fn pause(&self) -> Result<(), AireError> {
        self.tx.try_send(Command::Pause(self.id))
            .map_err(|_| AireError::CommandBufferFull)
    }

    /// Resumes playback.
    pub fn resume(&self) -> Result<(), AireError> {
        self.tx.try_send(Command::Resume(self.id))
            .map_err(|_| AireError::CommandBufferFull)
    }

    /// Stops playback and removes the sound from the mixer.
    pub fn stop(&self) -> Result<(), AireError> {
        self.tx.try_send(Command::Stop(self.id))
            .map_err(|_| AireError::CommandBufferFull)
    }

    /// Sets the volume in decibels.
    pub fn set_volume(&self, db: f32) -> Result<(), AireError> {
        self.tx.try_send(Command::SetVolume(self.id, db))
            .map_err(|_| AireError::CommandBufferFull)
    }

    /// Sets the pan from `0.0` (full left) to `1.0` (full right).
    pub fn set_pan(&self, pan: f32) -> Result<(), AireError> {
        self.tx.try_send(Command::SetPan(self.id, pan))
            .map_err(|_| AireError::CommandBufferFull)
    }
}
