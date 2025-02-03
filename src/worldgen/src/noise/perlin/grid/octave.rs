use crate::noise::perlin::grid::noise::GridNoise;
use crate::noise::perlin::noise::{LegacyNoise, Noise};
use crate::noise::perlin::octave;
use crate::noise::perlin::octave::{MultiOctaveNoise, MultiOctaveNoiseFactory, NoiseOctave};
use crate::rng::{RngForkable, RngPos};
use spherix_math::vector::Vector3f;

impl MultiOctaveNoiseFactory for MultiOctaveNoise<GridNoise> {
    fn create<R: RngForkable>(rng: &mut R, amplitudes: &Vec<f64>, first_octave: i32) -> Self {
        let len = amplitudes.len();

        let rng_pos = rng.fork_pos();
        let mut octaves = Vec::with_capacity(len);

        let mut persistence = Self::persistence(len as i32);

        for k in 0..len {
            if amplitudes[k] != 0.0 {
                let l = first_octave + k as i32;
                let mut rng_oct = rng_pos.by_hash(format!("octave_{}", l).to_owned());

                octaves.push(
                    NoiseOctave::new(
                        GridNoise::new(&mut rng_oct),
                        persistence * amplitudes[k],
                        octave::lacunarity(l),
                    )
                );
            }

            persistence /= 2.0;
        }

        Self {
            max_value: octave::edge_value(&octaves, 2.0),
            octaves,
        }
    }
}

impl MultiOctaveNoise<GridNoise> {
    #[inline]
    fn persistence(amplitude_i: i32) -> f64 {
        2.0f64.powi(amplitude_i - 1) / (2.0f64.powi(amplitude_i) - 1.0)
    }
}

impl Noise<Vector3f> for MultiOctaveNoise<GridNoise> {
    fn sample(&self, at: Vector3f) -> f64 {
        self.octaves
            .iter()
            .map(|octave| Noise::sample(octave, at))
            .sum()
    }
}

impl LegacyNoise<Vector3f> for MultiOctaveNoise<GridNoise> {
    fn sample(&self, at: Vector3f, y_amp: f64, y_min: f64) -> f64 {
        self.octaves
            .iter()
            .map(|octave| LegacyNoise::sample(octave, at, y_amp, y_min))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::noise::perlin::noise::Noise;
    use crate::noise::perlin::octave::{MultiOctaveNoise, MultiOctaveNoiseFactory};
    use crate::rng::XoroShiro;
    use spherix_math::vector::Vector3f;
    use spherix_util::assert_f64_eq;

    #[test]
    fn multi_octave_perlin_noise_sample() {
        let noise = MultiOctaveNoise::create(
            &mut XoroShiro::new(0x24B091C),
            &vec![1.0, 1.0],
            -4,
        );

        assert_eq!(2, noise.octaves.len());

        assert_f64_eq!(0.6666666666666666, noise.octaves[0].amplitude, 10);
        assert_f64_eq!(0.0625, noise.octaves[0].lacunarity, 10);

        assert_f64_eq!(0.3333333333333333, noise.octaves[1].amplitude, 10);
        assert_f64_eq!(0.125, noise.octaves[1].lacunarity, 10);

        assert_f64_eq!(2.0, noise.max_value, 10);

        assert_f64_eq!(-0.19834553595254156, noise.sample(Vector3f::new(-1.0, -2.0, 6.2)), 10);

        let noise = MultiOctaveNoise::create(
            &mut XoroShiro::new(0x24B091C),
            &vec![1.0, 1.0],
            -1,
        );

        assert_f64_eq!(0.09292965612437337, noise.sample(Vector3f::new(-1.0, -2.0, 6.2)), 10);
    }
}
