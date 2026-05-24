pub mod file;
pub mod oscillator;

/// A source of audio samples. Can be a file, a synthesized signal, or anything
/// that produces stereo output.
pub trait Source: Send {
    /// Fills `buffer` with stereo samples as `(left, right)` pairs.
    fn fill_buffer(&mut self, buffer: &mut [(f32, f32)]);
    /// Returns `true` when the source has no more samples to produce.
    fn is_finished(&self) -> bool;
}
