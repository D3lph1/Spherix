use crate::noise::perlin::grid::noise::GridNoise;
use crate::noise::perlin::noise::{LegacyNoise, Noise, SupremumNoise};
use crate::noise::perlin::octave;
use crate::noise::perlin::octave::{MultiOctaveNoiseFactory, NoiseOctave, NoiseOctaves};
use crate::rng::{Rng, RngForkable};
use core::panic;
use spherix_math::vector::Vector3f;
use std::collections::{BTreeSet, VecDeque};
use std::ops::RangeInclusive;

#[derive(Clone)]
pub struct LegacyMultiOctaveGridNoise
{
    octaves: NoiseOctaves<GridNoise>,
    max_value: f64,
}

impl MultiOctaveNoiseFactory for LegacyMultiOctaveGridNoise {
    fn create<R: Rng>(rng: &mut R, amplitudes: &Vec<f64>, mut first_octave: i32) -> Self {
        let len = amplitudes.len();

        if len == 0 {
            panic!("number of amplitudes must be greater than zero")
        }

        if first_octave > 0 {
            panic!("invalid first octave value")
        }

        let j = -first_octave;

        let mut octaves = VecDeque::with_capacity(len);
        let mut persistence = Self::persistence(len as i32);

        let noise = GridNoise::new(rng);
        if j >= 0 && j < len as i32 {
            if amplitudes[j as usize] != 0.0 {
                octaves.push_front(
                    NoiseOctave::new(
                        noise,
                        persistence * amplitudes[j as usize],
                        octave::lacunarity(first_octave + len as i32 - 1),
                    )
                );

                first_octave -= 1;
                persistence *= 2.0;
            }
        }

        for i1 in (0..j).rev() {
            if i1 < len as i32 {
                let amp = amplitudes[i1 as usize];
                if amp != 0.0 {
                    octaves.push_front(
                        NoiseOctave::new(
                            GridNoise::new(rng),
                            persistence * amp,
                            octave::lacunarity(first_octave + len as i32 - 1),
                        )
                    );

                    first_octave -= 1;
                    persistence *= 2.0;
                } else {
                    Self::skip_octave(rng);
                }
            } else {
                Self::skip_octave(rng);
            }
        }

        let octaves = octaves.into_iter().collect();

        Self {
            max_value: octave::edge_value(&octaves, 2.0),
            octaves,
        }
    }
}

impl LegacyMultiOctaveGridNoise {
    pub fn from_i32_amplitudes_range<R: RngForkable>(rng: &mut R, amp: RangeInclusive<i32>) -> Self {
        let (amplitudes, first_octave) = octave::make_amplitudes(
            BTreeSet::from_iter(amp.into_iter())
        );

        Self::create(rng, &amplitudes, first_octave)
    }

    pub fn octave(&self, i: usize) -> Option<&NoiseOctave<GridNoise>> {
        self.octaves.get(self.octaves.len() - 1 - i)
    }

    pub fn max_broken_value(&self, x: f64) -> f64 {
        octave::edge_value(&self.octaves, x + 2.0)
    }

    #[inline]
    fn skip_octave<R: Rng>(rng: &mut R) {
        rng.skip(262);
    }

    #[inline]
    fn persistence(amplitude_i: i32) -> f64 {
        1.0 / (2.0f64.powi(amplitude_i) - 1.0)
    }
}

impl Noise<Vector3f> for LegacyMultiOctaveGridNoise {
    fn sample(&self, at: Vector3f) -> f64 {
        self.octaves
            .iter()
            .map(|octave| Noise::sample(octave, at))
            .sum()
    }
}

impl LegacyNoise<Vector3f> for LegacyMultiOctaveGridNoise {
    fn sample(&self, at: Vector3f, y_amp: f64, y_min: f64) -> f64 {
        self.octaves
            .iter()
            .map(|octave| LegacyNoise::sample(octave, at, y_amp, y_min))
            .sum()
    }
}

impl SupremumNoise for LegacyMultiOctaveGridNoise {
    fn max_value(&self) -> f64 {
        self.max_value
    }
}


#[cfg(test)]
mod tests {
    use crate::noise::perlin::grid::legacy::LegacyMultiOctaveGridNoise;
    use crate::noise::perlin::noise::Noise;
    use crate::noise::perlin::octave::MultiOctaveNoiseFactory;
    use crate::rng::XoroShiro;
    use spherix_math::vector::Vector3f;
    use spherix_util::assert_f64_eq;

    #[test]
    fn legacy_multi_octave_perlin_noise_sample() {
        let noise = LegacyMultiOctaveGridNoise::create(
            &mut XoroShiro::new(0x24B091C),
            &vec![1.0, 1.0],
            -4,
        );

        assert_eq!(2, noise.octaves.len());

        assert_f64_eq!(0.6666666666666666, noise.octaves[0].amplitude, 10);
        assert_f64_eq!(0.0625, noise.octaves[0].lacunarity, 10);

        assert_f64_eq!(0.3333333333333333, noise.octaves[1].amplitude, 10);
        assert_f64_eq!(0.125, noise.octaves[1].lacunarity, 10);

        assert_f64_eq!(-0.35419610119465705, noise.sample(Vector3f::new(-1.0, -2.0, 6.2)), 10);

        let noise = LegacyMultiOctaveGridNoise::create(
            &mut XoroShiro::new(0x24B091C),
            &vec![1.0, 1.0],
            -1,
        );

        assert_f64_eq!(-0.12871201910867605, noise.sample(Vector3f::new(-1.0, -2.0, 6.2)), 10);

        let noise = LegacyMultiOctaveGridNoise::create(
            &mut XoroShiro::new(0x24B091C),
            &vec![0.5, 0.75],
            -1,
        );

        assert_f64_eq!(0.012496930953711655, noise.sample(Vector3f::new(14.0, -4.8, 11.8)), 10);

        let noise = LegacyMultiOctaveGridNoise::create(
            &mut XoroShiro::new(0x24B091C),
            &vec![0.5, 0.75],
            -4,
        );

        assert_f64_eq!(-0.1546099884442262, noise.sample(Vector3f::new(14.0, -4.8, 11.8)), 10);

        let noise = LegacyMultiOctaveGridNoise::create(
            &mut XoroShiro::new(0x24B091C),
            &vec![0.5, 0.75, 0.9],
            -7,
        );

        assert_f64_eq!(0.03837884770265749, noise.sample(Vector3f::new(5375010.0, 248195.5, -324117.4)), 10);
    }
}
