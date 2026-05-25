use crossbeam_channel::Sender;
use crate::error::AireError;
use crate::engine::{Command, SoundId};

/// A handle for controlling a sound after it has been added to the engine.
/// Obtained from [`crate::Engine::add_sound`]. All methods return
/// [`AireError::CommandBufferFull`] if the command queue is full.
pub struct SoundHandle {
    pub(crate) id: SoundId,
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

/// A handle for controlling a group of sounds by category (e.g. `"music"`, `"sfx"`).
/// Obtained from [`crate::Engine::group`]. All methods return [`AireError::CommandBufferFull`] if the
/// command queue is full.
#[derive(Clone)]
pub struct GroupHandle {
    name: String,
    tx: Sender<Command>,
}

impl GroupHandle {
    pub(crate) fn new(name: String, tx: Sender<Command>) -> Self {
        Self { name, tx }
    }

    /// Adds a sound to this group. Silently ignored if the sound has already finished.
    pub fn add(&self, sound: &SoundHandle) -> Result<(), AireError> {
        self.tx.try_send(Command::AddToGroup(sound.id, self.name.clone()))
            .map_err(|_| AireError::CommandBufferFull)
    }

    /// Sets the group volume in decibels, `0.0` is unity gain.
    pub fn set_volume(&self, db: f32) -> Result<(), AireError> {
        self.tx.try_send(Command::SetGroupVolume(self.name.clone(), db))
            .map_err(|_| AireError::CommandBufferFull)
    }

    /// Sets the group stereo balance from `0.0` (full left) to `1.0` (full right).
    /// `0.5` is center and is fully transparent. This is a balance (trim) control, not a pan position...
    /// individual sound panning is set via [`SoundHandle::set_pan`].
    pub fn set_pan(&self, pan: f32) -> Result<(), AireError> {
        self.tx.try_send(Command::SetGroupPan(self.name.clone(), pan.clamp(0.0, 1.0)))
            .map_err(|_| AireError::CommandBufferFull)
    }
}
