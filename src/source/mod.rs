pub mod file;

pub trait Source: Send {
    fn next_sample(&mut self) -> f32;
    fn is_finished(&self) -> bool;
}
