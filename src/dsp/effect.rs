/// A DSP effect that processes stereo samples. Implement this to create
/// custom effects and add them to a [`crate::Sound`] via [`crate::Sound::add_effect`].
pub trait Effect: Send {
    /// Processes a stereo sample and returns the result.
    fn process(&mut self, sample: (f32, f32)) -> (f32, f32);
    /// Returns `true` when the effect has finished. Defaults to `false`.
    fn is_finished(&self) -> bool {
        false
    }
}
