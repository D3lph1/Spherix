use crate::biome::noise::{FROZEN_TEMPERATURE_NOISE, INFO_NOISE, TEMPERATURE_NOISE};
use crate::noise::perlin::Noise;
use gxhash::GxBuildHasher;
use lru::LruCache;
use spherix_math::vector::{Vector2f, Vector3};
use spherix_world::chunk::biome::climate::{TemperatureCache, TemperatureModifier};
use spherix_world::chunk::biome::Biome;
use std::cell::RefCell;
use std::num::NonZeroUsize;

const TEMPERATURE_CACHE_SIZE: usize = 1024;


#[inline]
pub fn temperature(at: &Vector3, biome: &Biome) -> f32 {
    let t = biome.climate().temperature();

    cached_height_adjusted_temperature(at, t.value, t.modifier, &t.cache)
}

#[inline]
pub fn warm_enough_to_rain(at: &Vector3, biome: &Biome) -> bool {
    temperature(at, biome) >= 0.15
}

#[inline]
pub fn cold_enough_to_snow(at: &Vector3, biome: &Biome) -> bool {
    !warm_enough_to_rain(at, biome)
}

#[inline]
pub fn should_melt_frozen_ocean_iceberg_slightly(at: &Vector3, biome: &Biome) -> bool {
    temperature(at, biome) > 0.1
}

pub fn cached_height_adjusted_temperature(
    at: &Vector3,
    base_temperature: f32,
    modifier: TemperatureModifier,
    cache: &TemperatureCache,
) -> f32 {
    let mut map = cache
        .get_or(|| {
            RefCell::new(
                LruCache::with_hasher(
                    NonZeroUsize::new(TEMPERATURE_CACHE_SIZE).unwrap(),
                    GxBuildHasher::default()
                )
            )
        })
        .borrow_mut();
    let at_encoded: i64 = at.into();
    let temperature = map.get(&at_encoded);
    if temperature.is_some() {
        *temperature.unwrap()
    } else {
        let temperature = height_adjusted_temperature(at, base_temperature, modifier);
        map.put(at_encoded, temperature);

        temperature
    }
}

pub fn height_adjusted_temperature(at: &Vector3, base_temperature: f32, modifier: TemperatureModifier) -> f32 {
    let modified_temperature = modify_temperature(&at, base_temperature, modifier);
    if at.y > 80 {
        let f1 = (TEMPERATURE_NOISE.sample(Vector2f::new(at.x as f64 / 8.0, at.z as f64 / 8.0)) * 8.0) as f32;
        modified_temperature - (f1 + at.y as f32 - 80.0) * 0.05 / 40.0
    } else {
        modified_temperature
    }
}

fn modify_temperature(at: &Vector3, base_temperature: f32, modifier: TemperatureModifier) -> f32 {
    match modifier {
        TemperatureModifier::None => base_temperature,
        TemperatureModifier::Frozen => {
            let d0 = 7.0 * FROZEN_TEMPERATURE_NOISE.sample(Vector2f::new(at.x as f64 * 0.05, at.z as f64 * 0.05));
            let d1 = INFO_NOISE.sample(Vector2f::new(at.x as f64 * 0.2, at.z as f64 * 0.2));
            let d2 = d0 + d1;

            if d2 < 0.3 {
               let d3 = INFO_NOISE.sample(Vector2f::new(at.x as f64 * 0.09, at.z as f64 * 0.09));
                if d3 < 0.8 {
                    return 0.2
                }
            }

            base_temperature
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::biome::temperature::height_adjusted_temperature;
    use spherix_math::vector::Vector3;
    use spherix_util::assert_f32_eq;
    use spherix_world::chunk::biome::climate::TemperatureModifier;

    #[test]
    fn test_height_adjusted_temperature() {
        assert_f32_eq!(0.8, height_adjusted_temperature(&Vector3::new(152, -16, 471), 0.8, TemperatureModifier::None), 5);
        assert_f32_eq!(0.80425847, height_adjusted_temperature(&Vector3::new(152, 81, 471), 0.8, TemperatureModifier::None), 5);
        assert_f32_eq!(0.80300844, height_adjusted_temperature(&Vector3::new(152, 82, 471), 0.8, TemperatureModifier::None), 5);
        assert_f32_eq!(0.79300845, height_adjusted_temperature(&Vector3::new(152, 90, 471), 0.8, TemperatureModifier::None), 5);
        assert_f32_eq!(0.7680085, height_adjusted_temperature(&Vector3::new(152, 110, 471), 0.8, TemperatureModifier::None), 5);
        assert_f32_eq!(0.71800846, height_adjusted_temperature(&Vector3::new(152, 150, 471), 0.8, TemperatureModifier::None), 5);
        assert_f32_eq!(0.6555084, height_adjusted_temperature(&Vector3::new(152, 200, 471), 0.8, TemperatureModifier::None), 5);
        assert_f32_eq!(0.50675845, height_adjusted_temperature(&Vector3::new(152, 319, 471), 0.8, TemperatureModifier::None), 5);

        assert_f32_eq!(0.5, height_adjusted_temperature(&Vector3::new(152, -16, 471), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(0.2, height_adjusted_temperature(&Vector3::new(3, -16, 530), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(0.19019975, height_adjusted_temperature(&Vector3::new(3, 81, 530), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(0.18894975, height_adjusted_temperature(&Vector3::new(3, 82, 530), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(0.17894974, height_adjusted_temperature(&Vector3::new(3, 90, 530), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(0.15394975, height_adjusted_temperature(&Vector3::new(3, 110, 530), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(0.10394974, height_adjusted_temperature(&Vector3::new(3, 150, 530), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(0.04144974, height_adjusted_temperature(&Vector3::new(3, 200, 530), 0.5, TemperatureModifier::Frozen), 5);
        assert_f32_eq!(-0.10730027, height_adjusted_temperature(&Vector3::new(3, 319, 530), 0.5, TemperatureModifier::Frozen), 5);
    }
}
