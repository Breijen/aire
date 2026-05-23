use crossbeam_channel::Sender;
use crate::error::AireError;
use crate::engine::{Command, SoundId};

pub struct SoundHandle {
    id: SoundId,
    tx: Sender<Command>,
}

impl SoundHandle {
    pub(crate) fn new(id: SoundId, tx: Sender<Command>) -> Self {
        Self { id, tx }
    }

    pub fn pause(&self) -> Result<(), AireError> {
        self.tx.try_send(Command::Pause(self.id))
            .map_err(|_| AireError::CommandBufferFull)
    }

    pub fn resume(&self) -> Result<(), AireError> {
        self.tx.try_send(Command::Resume(self.id))
            .map_err(|_| AireError::CommandBufferFull)
    }

    pub fn stop(&self) -> Result<(), AireError> {
        self.tx.try_send(Command::Stop(self.id))
            .map_err(|_| AireError::CommandBufferFull)
    }
}
