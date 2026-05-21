pub trait Effect: Send {
    fn process(&mut self, sample: (f32, f32)) -> (f32, f32);
    fn is_finished(&self) -> bool {
        false
    }
}
