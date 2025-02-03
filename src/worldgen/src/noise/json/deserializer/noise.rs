use crate::noise::density::density::DensityFunctions;
use crate::noise::density::noise::{BlendDensity, OldBlendedNoise, ShiftA, ShiftB};
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use crate::rng::XoroShiro;
use serde_json::Value;

pub struct BlendedNoiseDeserializer;

impl Deserializer<DensityFunctions> for BlendedNoiseDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::BlendDensity(
            Box::new(
                BlendDensity::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct OldBlendedNoiseDeserializer;

impl Deserializer<DensityFunctions> for OldBlendedNoiseDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        let xz_scale = resolver.resolve_field(json, "xz_scale")?;
        let y_scale = resolver.resolve_field(json, "y_scale")?;
        let xz_factor = resolver.resolve_field(json, "xz_factor")?;
        let y_factor = resolver.resolve_field(json, "y_factor")?;
        let smear_scale_multiplier = resolver.resolve_field(json, "smear_scale_multiplier")?;

        Ok(DensityFunctions::OldBlendedNoise(
            OldBlendedNoise::create(
                &mut XoroShiro::new(0),
                xz_scale,
                y_scale,
                xz_factor,
                y_factor,
                smear_scale_multiplier,
            )
        ))
    }
}

pub struct ShiftADeserializer;

impl Deserializer<DensityFunctions> for ShiftADeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::ShiftA(
            ShiftA::new(
                resolver.resolve_field(json, "argument")?
            )
        ))
    }
}

pub struct ShiftBDeserializer;

impl Deserializer<DensityFunctions> for ShiftBDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::ShiftB(ShiftB::new(
            resolver.resolve_field(json, "argument")?
        )))
    }
}
