use crate::noise::density::density::{DensityFunction, DensityFunctions};
use crate::noise::json::value_resolver::ValueResolver;
use crate::noise::perlin::octave::MultiOctaveNoiseFactory;
use crate::rng::Rng;
use std::collections::HashMap;

use crate::noise::json::deserializer::blend::{BlendAlphaDeserializer, BlendOffsetDeserializer};
use crate::noise::json::deserializer::maker::{InterpolatedDeserializer, MarkerDeserializer};
use crate::noise::json::deserializer::misc::{ClampDeserializer, RangeChoiceDeserializer, WeirdScaledSamplerDeserializer, YClampedGradientDeserializer};
use crate::noise::json::deserializer::noise::{BlendedNoiseDeserializer, OldBlendedNoiseDeserializer, ShiftADeserializer, ShiftBDeserializer};
use crate::noise::json::deserializer::spline::SplineDeserializer;
use crate::noise::json::deserializer::unary::{AbsDeserializer, CubeDeserializer, HalfNegativeDeserializer, QuarterNegativeDeserializer, SquareDeserializer, SqueezeDeserializer};
use crate::noise::json::deserializer::{AddDeserializer, Cache2DDeserializer, CacheOnceDeserializer, Deserializer, FlatCacheDeserializer, MaxDeserializer, MinDeserializer, MulDeserializer, NoiseDeserializer, ShiftedNoiseDeserializer};
pub use resolver::Resolver;

pub mod resolver;
pub mod resolvable;
pub mod value_resolver;
pub mod deserializer;

pub fn deserializers() -> HashMap<String, Box<dyn Deserializer<DensityFunctions>>> {
    HashMap::from([
        ("minecraft:cache_2d".to_owned(), cast(Cache2DDeserializer)),
        ("minecraft:flat_cache".to_owned(), cast(FlatCacheDeserializer)),
        ("minecraft:shifted_noise".to_owned(), cast(ShiftedNoiseDeserializer)),
        ("minecraft:noise".to_owned(), cast(NoiseDeserializer)),
        ("minecraft:continentalness".to_owned(), cast(NoiseDeserializer)),
        ("minecraft:old_blended_noise".to_owned(), cast(OldBlendedNoiseDeserializer)),
        ("minecraft:interpolated".to_owned(), cast(InterpolatedDeserializer)),
        ("minecraft:add".to_owned(), cast(AddDeserializer)),
        ("minecraft:mul".to_owned(), cast(MulDeserializer)),
        ("minecraft:min".to_owned(), cast(MinDeserializer)),
        ("minecraft:max".to_owned(), cast(MaxDeserializer)),
        ("minecraft:abs".to_owned(), cast(AbsDeserializer)),
        ("minecraft:squeeze".to_owned(), cast(SqueezeDeserializer)),
        ("minecraft:square".to_owned(), cast(SquareDeserializer)),
        ("minecraft:cube".to_owned(), cast(CubeDeserializer)),
        ("minecraft:half_negative".to_owned(), cast(HalfNegativeDeserializer)),
        ("minecraft:blend_density".to_owned(), cast(BlendedNoiseDeserializer)),
        ("minecraft:y_clamped_gradient".to_owned(), cast(YClampedGradientDeserializer)),
        ("minecraft:range_choice".to_owned(), cast(RangeChoiceDeserializer)),
        ("minecraft:quarter_negative".to_owned(), cast(QuarterNegativeDeserializer)),
        ("minecraft:blend_offset".to_owned(), cast(BlendOffsetDeserializer)),
        ("minecraft:blend_alpha".to_owned(), cast(BlendAlphaDeserializer)),
        ("minecraft:cache_once".to_owned(), cast(CacheOnceDeserializer)),
        ("minecraft:spline".to_owned(), cast(SplineDeserializer)),
        ("minecraft:shift_a".to_owned(), cast(ShiftADeserializer)),
        ("minecraft:shift_b".to_owned(), cast(ShiftBDeserializer)),
        ("minecraft:clamp".to_owned(), cast(ClampDeserializer)),
        ("minecraft:weird_scaled_sampler".to_owned(), cast(WeirdScaledSamplerDeserializer)),
    ])
}

pub fn deserializers_with_markers() -> HashMap<String, Box<dyn Deserializer<DensityFunctions>>> {
    let mut deserializers = deserializers();
    deserializers.insert("minecraft:interpolated".to_owned(), cast(MarkerDeserializer::new("interpolated")));
    deserializers.insert("minecraft:cache_2d".to_owned(), cast(MarkerDeserializer::new("cache_2d")));
    deserializers.insert("minecraft:flat_cache".to_owned(), cast(MarkerDeserializer::new("flat_cache")));
    deserializers.insert("minecraft:cache_once".to_owned(), cast(MarkerDeserializer::new("cache_once")));

    deserializers
}

fn cast<T: Deserializer<DensityFunctions> + 'static>(t: T) -> Box<dyn Deserializer<DensityFunctions>> {
    Box::new(t)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;

    use spherix_math::vector::Vector3;
    use spherix_util::assert_f64_eq;

    use crate::noise::density::density::{DensityFunction, DensityFunctionContext, SetupNoiseMapper};
    use crate::noise::json::cast;
    use crate::noise::json::deserializer::binary::AddDeserializer;
    use crate::noise::json::deserializer::{Cache2DDeserializer, NoiseDeserializer, ShiftedNoiseDeserializer};
    use crate::noise::json::resolver::Resolver;
    use crate::noise::json::value_resolver::MockValueResolver;
    use crate::rng::{RngForkable, XoroShiro};

    #[test]
    fn resolver_resolve() {
        let resolver = Resolver::new(
            HashMap::from([
                ("minecraft:cache_2d".to_owned(), cast(Cache2DDeserializer)),
                ("minecraft:shifted_noise".to_owned(), cast(ShiftedNoiseDeserializer)),
                ("minecraft:continentalness".to_owned(), cast(NoiseDeserializer)),
                ("minecraft:add".to_owned(), cast(AddDeserializer)),
            ]),
            Box::new(MockValueResolver::new(HashMap::from([
                ("minecraft:shifted_noise".to_owned(), json!({
                    "noise": "minecraft:continentalness",
                    "shift_x": 1.0,
                    "shift_y": 1.0,
                    "shift_z": 1.0,
                    "xz_scale": 1.0,
                    "y_scale": 1.0,
                })),
                ("minecraft:continentalness".to_owned(), json!({
                  "amplitudes": [
                    1.0, 1.0, 2.0, 2.0, 2.0,
                    1.0, 1.0, 1.0, 1.0
                  ],
                  "firstOctave": -9
                })),
            ]))),
        );

        let json = json!({
            "type": "minecraft:cache_2d",
            "argument": {
                "type": "minecraft:add",
                "argument1": 10.0,
                "argument2": "minecraft:shifted_noise"
            }
        });

        let mut rng = XoroShiro::new(0x41A00F);

        let resolved = resolver.resolve(&json);
        let df = resolved.unwrap();

        let df = df.map(&SetupNoiseMapper::new(Arc::new(rng.fork_pos())));

        assert_f64_eq!(
            9.798801511251833,
            df.sample(Vector3::new(2, 1, -7), &mut DensityFunctionContext::default()),
            10
        );

        assert_f64_eq!(
            10.7733977030009,
            df.sample(Vector3::new(4150, -21, -700), &mut DensityFunctionContext::default()),
            10
        );
    }
}
