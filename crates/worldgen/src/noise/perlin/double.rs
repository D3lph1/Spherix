use crate::noise::perlin::noise::{LegacyNoise, Noise, SupremumNoise};
use crate::noise::perlin::octave::MultiOctaveNoiseFactory;
use crate::rng::{Rng, RngForkable};
use spherix_math::vector::Vector3f;

#[derive(Clone)]
pub struct DoubleMultiOctavePerlinNoise<N>
where
    N: Clone
{
    first: N,
    second: N,
    value_factor: f64,
    max_value: f64,
}

impl<N> Noise<Vector3f> for DoubleMultiOctavePerlinNoise<N>
where 
    N: Noise<Vector3f> + SupremumNoise + Clone
{
    fn sample(&self, at: Vector3f) -> f64 {
        let v = at * Self::POS_MULTIPLIER;

        (self.first.sample(at) + self.second.sample(v)) * self.value_factor
    }
}

impl<N> LegacyNoise<Vector3f> for DoubleMultiOctavePerlinNoise<N>
where
    N: LegacyNoise<Vector3f> + SupremumNoise + Clone
{
    fn sample(&self, at: Vector3f, y_amp: f64, y_min: f64) -> f64 {
        let v = at * Self::POS_MULTIPLIER;

        (self.first.sample(at, y_amp, y_min) + self.second.sample(v, y_amp, y_min)) * self.value_factor
    }
}

impl<N> SupremumNoise for DoubleMultiOctavePerlinNoise<N>
where
    N: SupremumNoise + Clone
{
    fn max_value(&self) -> f64 {
        self.max_value
    }
}

impl <N> MultiOctaveNoiseFactory for DoubleMultiOctavePerlinNoise<N>
where
    N: MultiOctaveNoiseFactory + SupremumNoise + Clone
{
    fn create<R: RngForkable>(rng: &mut R, amplitudes: &Vec<f64>, first_octave: i32) -> Self {
        let first = N::create(rng, amplitudes, first_octave);
        let second = N::create(rng, amplitudes, first_octave);

        let mut j = i32::MAX;
        let mut k = i32::MIN;

        for (i, amp) in amplitudes.iter().enumerate() {
            if *amp != 0.0 {
                let i = i as i32;
                j = j.min(i);
                k = k.max(i);
            }
        }

        Self::new(
            first,
            second,
            0.16666666666666666 / Self::expected_deviation(k - j)
        )
    }
}

impl <N> DoubleMultiOctavePerlinNoise<N>
where
    N: SupremumNoise + Clone
{
    const POS_MULTIPLIER: f64 = 1.0181268882175227;
    
    pub fn new(first: N, second: N, value_factor: f64) -> Self {
        Self {
            value_factor,
            max_value: (first.max_value() + second.max_value()) * value_factor,
            first,
            second,
        }
    }

    fn expected_deviation(x: i32) -> f64 {
        0.1 * (1.0 + 1.0 / (x + 1) as f64)
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::perlin::double::DoubleMultiOctavePerlinNoise;
    use crate::noise::perlin::noise::Noise;
    use crate::noise::perlin::octave::{MultiOctaveNoise, MultiOctaveNoiseFactory};
    use crate::noise::perlin::GridNoise;
    use crate::rng::{RngForkable, RngPos, XoroShiro};
    use spherix_math::vector::Vector3f;
    use spherix_util::assert_f64_eq;

    #[test]
    fn double_multi_octave_perlin_noise1() {
        let noise = DoubleMultiOctavePerlinNoise::<MultiOctaveNoise<GridNoise>>::create(
            &mut XoroShiro::new(0x8C190F14101),
            &vec![0.64, 0.21, 0.58, 1.0],
            -5
        );

        assert_f64_eq!(2.887111111111111, noise.max_value, 10);

        let s = noise.sample(Vector3f::new(-2.18, 0.77, 4.95));

        assert_f64_eq!(0.16774247552182325, s, 10);
    }

    #[test]
    fn double_multi_octave_perlin_noise2() {
        let mut rng = XoroShiro::new(1);
        let rng_pos = rng.fork_pos();

        let noise = DoubleMultiOctavePerlinNoise::<MultiOctaveNoise<GridNoise>>::create(
            &mut rng_pos.by_hash("minecraft:erosion".to_owned()),
            &vec![1.0, 1.0, 0.0, 1.0, 1.0],
            -9
        );

        assert_f64_eq!(4.838709677419354, noise.max_value, 10);

        let s = noise.sample(Vector3f::new(28.887526207138677, 0.0, 45.572859785704942));

        assert_f64_eq!(-0.21006495645374587, s, 10);
    }
}
