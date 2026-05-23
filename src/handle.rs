use std::sync::{Arc, Mutex};
use rtrb::Producer;
use crate::engine::{Command, SoundId};

pub struct SoundHandle {
    id: SoundId,
    tx: Arc<Mutex<Producer<Command>>>,
}

impl SoundHandle {
    pub(crate) fn new(id: SoundId, tx: Arc<Mutex<Producer<Command>>>) -> Self {
        Self { id, tx }
    }

    pub fn pause(&self) {
        self.tx.lock().unwrap().push(Command::Pause(self.id))
            .unwrap_or_else(|_| panic!("command buffer full"));
    }

    pub fn resume(&self) {
        self.tx.lock().unwrap().push(Command::Resume(self.id))
            .unwrap_or_else(|_| panic!("command buffer full"));
    }
}
