use crate::noise::density::density::DensityFunctions;
use crate::noise::density::misc::{Clamp, RangeChoice, WeirdScaledSampler, YClampedGradient};
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use serde_json::Value;

pub struct YClampedGradientDeserializer;

impl Deserializer<DensityFunctions> for YClampedGradientDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::YClampedGradient(
            YClampedGradient::new(
                resolver.resolve_field(json, "from_y")?,
                resolver.resolve_field(json, "to_y")?,
                resolver.resolve_field(json, "from_value")?,
                resolver.resolve_field(json, "to_value")?,
            )
        ))
    }
}

pub struct RangeChoiceDeserializer;

impl Deserializer<DensityFunctions> for RangeChoiceDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::RangeChoice(
            Box::new(
                RangeChoice::new(
                    resolver.resolve_field(json, "input")?,
                    resolver.resolve_field(json, "min_inclusive")?,
                    resolver.resolve_field(json, "max_exclusive")?,
                    resolver.resolve_field(json, "when_in_range")?,
                    resolver.resolve_field(json, "when_out_of_range")?,
                )
            )
        ))
    }
}

pub struct ClampDeserializer;

impl Deserializer<DensityFunctions> for ClampDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Clamp(
            Box::new(
                Clamp::new(
                    resolver.resolve_field(json, "input")?,
                    resolver.resolve_field(json, "min")?,
                    resolver.resolve_field(json, "max")?,
                )
            )
        ))
    }
}

pub struct WeirdScaledSamplerDeserializer;

impl Deserializer<DensityFunctions> for WeirdScaledSamplerDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::WeirdScaledSampler(
            Box::new(
                WeirdScaledSampler::new(
                    resolver.resolve_field(json, "input")?,
                    resolver.resolve_field(json, "noise")?,
                    resolver.resolve_field(json, "rarity_value_mapper")?,
                )
            )
        ))
    }
}
