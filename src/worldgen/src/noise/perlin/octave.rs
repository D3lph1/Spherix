use crate::noise::perlin::noise::{LegacyNoise, Noise, SupremumNoise};
use crate::rng::RngForkable;
use spherix_math::vector::{Vector2f, Vector3f};
use std::collections::BTreeSet;

pub type NoiseOctaves<N> = Vec<NoiseOctave<N>>;

/// Read more about multi-octave perlin noise in the [`article`].
/// 
/// [`article`]: https://catlikecoding.com/unity/tutorials/pseudorandom-noise/noise-variants/
#[derive(Clone)]
pub struct NoiseOctave<N>
where
    N: Clone,
{
    pub noise: N,
    /// Amplitude with reduction
    pub amplitude: f64,
    /// Lacunarity defines frequency reduction
    pub lacunarity: f64,
}

impl<N> NoiseOctave<N>
where
    N: Clone,
{
    #[inline]
    pub fn new(noise: N, amplitude: f64, lacunarity: f64) -> Self {
        Self {
            noise,
            amplitude,
            lacunarity,
        }
    }

    #[inline]
    pub fn inner(&self) -> &N {
        &self.noise
    }
}

impl<N> Noise<Vector3f> for NoiseOctave<N>
where
    N: Noise<Vector3f> + Clone,
{
    fn sample(&self, at: Vector3f) -> f64 {
        self.amplitude * self.noise.sample(wrap_vector(at * self.lacunarity))
    }
}

impl<N> Noise<Vector2f> for NoiseOctave<N>
where
    N: Noise<Vector2f> + Clone,
{
    fn sample(&self, at: Vector2f) -> f64 {
        self.amplitude * self.noise.sample(wrap_vector_2(at * self.lacunarity))
    }
}

impl<N> LegacyNoise<Vector3f> for NoiseOctave<N>
where
    N: LegacyNoise<Vector3f> + Clone,
{
    fn sample(&self, at: Vector3f, y_amp: f64, y_min: f64) -> f64 {
        self.amplitude * self.noise.sample(
            wrap_vector(at * self.lacunarity),
            y_amp * self.lacunarity,
            y_min * self.lacunarity,
        )
    }
}

pub trait MultiOctaveNoiseFactory {
    fn create<R: RngForkable>(rng: &mut R, amplitudes: &Vec<f64>, first_octave: i32) -> Self;
}

#[derive(Clone)]
pub struct MultiOctaveNoiseParameters {
    pub first_octave: i32,
    pub amplitudes: Vec<f64>,
}

impl MultiOctaveNoiseParameters {
    #[inline]
    pub fn new(first_octave: i32, amplitudes: Vec<f64>) -> Self {
        Self {
            first_octave,
            amplitudes,
        }
    }
}

#[derive(Clone)]
pub struct MultiOctaveNoise<N>
where
    N: Clone,
{
    pub(crate) octaves: NoiseOctaves<N>,
    pub(crate) max_value: f64,
}

impl<N> SupremumNoise for MultiOctaveNoise<N>
where
    N: Clone
{
    fn max_value(&self) -> f64 {
        self.max_value
    }
}

pub fn edge_value<N>(octaves: &NoiseOctaves<N>, multiplier: f64) -> f64
where
    N: Clone
{
    octaves
        .iter()
        .map(|oct| oct.amplitude * multiplier)
        .sum()
}

#[inline]
pub fn lacunarity(octave: i32) -> f64 {
    2.0f64.powi(octave)
}

#[inline]
pub fn wrap_vector(pos: Vector3f) -> Vector3f {
    Vector3f::new(
        wrap(pos.x),
        wrap(pos.y),
        wrap(pos.z),
    )
}

pub fn wrap_vector_2(pos: Vector2f) -> Vector2f {
    Vector2f::new(
        wrap(pos.x),
        wrap(pos.z()),
    )
}


#[inline]
pub fn wrap(x: f64) -> f64 {
    x - (x / 3.3554432E7 + 0.5).floor() * 3.3554432E7
}

pub fn make_amplitudes(set: BTreeSet<i32>) -> (Vec<f64>, i32) {
    if set.len() == 0 {
        panic!("Empty set is not allowed")
    }

    let i = -set.first().unwrap();
    let j = *set.last().unwrap();
    let k = i + j + 1;

    if k < 1 {
        panic!("Total number of octaves needs to be greater or equals than 1")
    }

    let mut v = vec![0.0; k as usize];

    for x in set {
        v[(x + i) as usize] = 1.0;
    }

    (v, -i)
}
