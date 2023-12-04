pub fn in_sine(val: f32) -> f32 {
    use std::f32::consts::PI;
    1.0 - f32::cos((val * PI) / 2.0)
}
