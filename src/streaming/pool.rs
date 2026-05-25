use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{unbounded, Receiver, Sender};
use ringbuf::{HeapCons, HeapProd, HeapRb};
use ringbuf::traits::{Observer, Producer, Split};

use super::StreamDecoder;

const RING_BUF_SIZE: usize = 65_536;
const CHUNK_SIZE: usize = 2_048;

struct StreamTask {
    decoder: Box<dyn StreamDecoder>,
    producer: HeapProd<f32>,
    looping: Arc<AtomicBool>,
    finished: Arc<AtomicBool>,
}

pub struct DecodePool {
    senders: Vec<Sender<StreamTask>>,
    next: AtomicUsize,
    _threads: Vec<JoinHandle<()>>,
}

impl DecodePool {
    pub fn new(num_threads: usize) -> Self {
        assert!(num_threads > 0, "DecodePool needs at least one thread");

        let mut senders = Vec::with_capacity(num_threads);
        let mut threads = Vec::with_capacity(num_threads);

        for _ in 0..num_threads {
            let (tx, rx) = unbounded::<StreamTask>();
            senders.push(tx);
            threads.push(thread::spawn(move || worker(rx)));
        }

        Self { senders, next: AtomicUsize::new(0), _threads: threads }
    }

    /// Creates a ring buffer, registers the decoder with a worker thread,
    /// and returns finished flag to the caller.
    pub(crate) fn register(
        &self,
        mut decoder: Box<dyn StreamDecoder>,
        looping: Arc<AtomicBool>,
    ) -> (HeapCons<f32>, Arc<AtomicBool>) {
        let (mut producer, consumer) = HeapRb::<f32>::new(RING_BUF_SIZE).split();
        let finished = Arc::new(AtomicBool::new(false));
        
        let mut scratch = Vec::with_capacity(CHUNK_SIZE);
        while producer.occupied_len() < RING_BUF_SIZE / 2 && !decoder.finished() {
            scratch.clear();
            if decoder.decode_chunk(&mut scratch, CHUNK_SIZE).is_err() { break; }
            producer.push_slice(&scratch);
        }

        let task = StreamTask { decoder, producer, looping, finished: finished.clone() };

        let idx = self.next.fetch_add(1, Ordering::Relaxed) % self.senders.len();
        self.senders[idx].send(task).ok();

        (consumer, finished)
    }
}

fn worker(rx: Receiver<StreamTask>) {
    let mut tasks: Vec<StreamTask> = Vec::new();
    let mut scratch: Vec<f32> = Vec::with_capacity(CHUNK_SIZE);

    loop {
        while let Ok(task) = rx.try_recv() {
            tasks.push(task);
        }

        let mut did_work = false;

        for task in &mut tasks {
            if task.finished.load(Ordering::Relaxed) { continue; }
            if task.producer.vacant_len() < CHUNK_SIZE { continue; }

            scratch.clear();

            match task.decoder.decode_chunk(&mut scratch, CHUNK_SIZE) {
                Err(_) => { task.finished.store(true, Ordering::Relaxed); continue; }
                Ok(_)  => { task.producer.push_slice(&scratch); did_work = true; }
            }

            if task.decoder.finished() {
                if task.looping.load(Ordering::Relaxed) {
                    if task.decoder.reset().is_err() {
                        task.finished.store(true, Ordering::Relaxed);
                    }
                } else {
                    task.finished.store(true, Ordering::Relaxed);
                }
            }
        }

        tasks.retain(|t| !t.finished.load(Ordering::Relaxed));

        if !did_work {
            thread::sleep(Duration::from_millis(2));
        }
    }
}
