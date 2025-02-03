/// 3-dimensional noise contract
pub trait Noise<V> {
    fn sample(&self, at: V) -> f64;
}

pub trait LegacyNoise<V> {
    fn sample(&self, at: V, y_amp: f64, y_min: f64) -> f64;
}

/// 3-dimensional noise with upper limit
pub trait SupremumNoise {
    fn max_value(&self) -> f64;
}
