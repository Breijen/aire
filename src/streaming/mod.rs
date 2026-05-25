use std::error::Error;

pub(crate) mod decoder;
pub(crate) mod pool;

pub trait StreamDecoder: Send {
    fn channels(&self) -> usize;
    fn sample_rate(&self) -> u32;
    fn decode_chunk(&mut self, out: &mut Vec<f32>, target: usize) -> Result<usize, Box<dyn Error + Send + Sync>>;
    fn reset(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>;
    fn finished(&self) -> bool;
}
