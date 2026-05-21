pub fn convert_db(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

pub fn milliseconds_to_samples(milliseconds: u32, sample_rate: u32) -> u32 {
    (milliseconds * sample_rate) / 1000
}