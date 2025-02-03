use crate::noise::perlin::noise::Noise;
use crate::noise::perlin::octave::{MultiOctaveNoiseFactory, NoiseOctave, NoiseOctaves};
use crate::noise::perlin::simplex::noise::SimplexNoise;
use crate::rng::{Rng, RngForkable};
use spherix_math::vector::Vector2f;

#[derive(Clone)]
pub struct MultiOctaveNoise<N>
where
    N: Clone,
{
    octaves: NoiseOctaves<N>
}

impl<N> MultiOctaveNoise<N>
where
    N: Clone,
{
    #[inline]
    fn skip_octave<R: Rng>(rng: &mut R) {
        rng.skip(262);
    }

    #[inline]
    fn persistence(amplitude_i: i32) -> f64 {
        1.0 / (2.0f64.powi(amplitude_i) - 1.0)
    }
}

impl MultiOctaveNoiseFactory for MultiOctaveNoise<SimplexNoise> {
    fn create<R: RngForkable>(rng: &mut R, amplitudes: &Vec<f64>, mut first_octave: i32) -> Self {
        let len = amplitudes.len();

        if len == 0 {
            panic!("number of amplitudes must be greater than zero")
        }

        if first_octave > 0 {
            panic!("invalid first octave value")
        }

        let mut persistence = Self::persistence(len as i32);
        let mut lacunarity = 1.0;

        let noise = SimplexNoise::new(rng);

        let mut octaves = Vec::with_capacity(len);
        for i in 0..len {
            if i == 0 {
                octaves.push(
                    NoiseOctave::new(
                        noise.clone(),
                        persistence * amplitudes[i],
                        lacunarity,
                    )
                )
            } else {
                octaves.push(
                    NoiseOctave::new(
                        SimplexNoise::new(rng),
                        persistence * amplitudes[i],
                        lacunarity,
                    )
                );
            }

            persistence *= 2.0;
            lacunarity /= 2.0;
        }

        let octaves = octaves.into_iter().collect();

        Self {
            octaves,
        }
    }
}

impl Noise<Vector2f> for MultiOctaveNoise<SimplexNoise> {
    fn sample(&self, at: Vector2f) -> f64 {
        self.octaves
            .iter()
            .map(|octave| Noise::<Vector2f>::sample(octave, at))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::perlin::noise::Noise;
    use crate::noise::perlin::simplex::octave::{MultiOctaveNoise, MultiOctaveNoiseFactory};
    use crate::rng::{LcgEntropySrc, U32EntropySrc, U32EntropySrcRng, XoroShiro};
    use spherix_math::vector::Vector2f;
    use spherix_util::assert_f64_eq;

    #[test]
    fn multi_octave_noise_simplex_with_xoroshiro_rng() {
        let noise = MultiOctaveNoise::create(
            &mut XoroShiro::new(0),
            &vec![1.0],
            0
        );

        assert_f64_eq!(-0.23822674514792452, noise.sample(Vector2f::new(174.0, 241.0)), 10);

        let n = MultiOctaveNoise::create(
            &mut XoroShiro::new(0),
            &vec![1.0, 1.0, 1.0],
            -2
        );

        assert_f64_eq!(0.4011425454935378, n.sample(Vector2f::new(174.0, 241.0)), 10);
    }

    #[test]
    fn multi_octave_noise_simplex_with_legacy_rng() {
        let noise = MultiOctaveNoise::create(
            &mut U32EntropySrcRng::new(LcgEntropySrc::new(1234)),
            &vec![1.0],
            0
        );

        assert_f64_eq!(0.0, noise.sample(Vector2f::new(0.0, 0.0)), 10);
        assert_f64_eq!(-0.23822674514792452, noise.sample(Vector2f::new(174.0, 241.0)), 10);
        assert_f64_eq!(0.12739715248543546, noise.sample(Vector2f::new(-508.17, 1263.85)), 10);

        let noise = MultiOctaveNoise::create(
            &mut U32EntropySrcRng::new(LcgEntropySrc::new(3456)),
            &vec![1.0, 1.0, 1.0],
            -2
        );

        assert_f64_eq!(0.0, noise.sample(Vector2f::new(0.0, 0.0)), 10);
        assert_f64_eq!(0.14143656067241273, noise.sample(Vector2f::new(174.0, 241.0)), 10);
        assert_f64_eq!(0.02413724998481663, noise.sample(Vector2f::new(-508.17, 1263.85)), 10);
    }
}
