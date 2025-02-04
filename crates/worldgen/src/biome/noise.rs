use crate::noise::perlin::octave::MultiOctaveNoiseFactory;
use crate::noise::perlin::simplex::noise::SimplexNoise;
use crate::noise::perlin::SimplexMultiOctaveNoise;
use crate::rng::{LcgEntropySrc, U32EntropySrc, U32EntropySrcRng};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref TEMPERATURE_NOISE: SimplexMultiOctaveNoise<SimplexNoise> = {
        SimplexMultiOctaveNoise::create(
            &mut U32EntropySrcRng::new(LcgEntropySrc::new(1234)),
            &vec![1.0],
            0
        )
    };

    pub static ref FROZEN_TEMPERATURE_NOISE: SimplexMultiOctaveNoise<SimplexNoise> = {
        SimplexMultiOctaveNoise::create(
            &mut U32EntropySrcRng::new(LcgEntropySrc::new(3456)),
            &vec![1.0, 1.0, 1.0],
            -2
        )
    };

    pub static ref INFO_NOISE: SimplexMultiOctaveNoise<SimplexNoise> = {
        SimplexMultiOctaveNoise::create(
            &mut U32EntropySrcRng::new(LcgEntropySrc::new(2345)),
            &vec![1.0],
            0
        )
    };
}
