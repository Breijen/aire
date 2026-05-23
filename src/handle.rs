use crossbeam_channel::Sender;
use crate::engine::{Command, SoundId};

pub struct SoundHandle {
    id: SoundId,
    tx: Sender<Command>,
}

impl SoundHandle {
    pub(crate) fn new(id: SoundId, tx: Sender<Command>) -> Self {
        Self { id, tx }
    }

    pub fn pause(&self) {
        self.tx.try_send(Command::Pause(self.id))
            .unwrap_or_else(|_| panic!("command buffer full"));
    }

    pub fn resume(&self) {
        self.tx.try_send(Command::Resume(self.id))
            .unwrap_or_else(|_| panic!("command buffer full"));
    }

    pub fn stop(&self) {
        self.tx.try_send(Command::Stop(self.id))
            .unwrap_or_else(|_| panic!("command buffer full"));
    }
}
