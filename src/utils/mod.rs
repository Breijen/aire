pub fn convert_db(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}