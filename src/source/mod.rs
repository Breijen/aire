pub mod file;

/// A source of audio samples. Can be a file, a synthesized signal, or anything
/// that produces stereo output sample by sample.
pub trait Source: Send {
    /// Returns the next stereo sample as `(left, right)`.
    fn next_sample(&mut self) -> (f32, f32);
    /// Returns `true` when the source has no more samples to produce.
    fn is_finished(&self) -> bool;
}
