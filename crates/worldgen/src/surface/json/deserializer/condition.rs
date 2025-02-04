use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use crate::surface::condition_factory::{AbovePreliminaryConditionFactory, BiomeConditionFactory, ConditionFactories, HoleConditionFactory, NoiseThresholdConditionFactory, NotConditionFactory, SteepMaterialConditionFactory, StoneDepthConditionFactory, TemperatureConditionFactory, VerticalGradientConditionFactory, WaterConditionFactory, YAboveConditionFactory};
use serde_json::Value;
use std::sync::Arc;

pub struct AbovePreliminaryDeserializer;

impl Deserializer<ConditionFactories> for AbovePreliminaryDeserializer {
    fn deserialize(&self, _: &Value, _: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(ConditionFactories::AbovePreliminary(AbovePreliminaryConditionFactory))
    }
}

pub struct BiomeDeserializer;

impl Deserializer<ConditionFactories> for BiomeDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(
            ConditionFactories::Biome(
                BiomeConditionFactory {
                    biome_is: Arc::new(resolver.resolve_field(json, "biome_is")?)
                }
            )
        )
    }
}

pub struct HoleDeserializer;

impl Deserializer<ConditionFactories> for HoleDeserializer {
    fn deserialize(&self, _: &Value, _: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(ConditionFactories::Hole(HoleConditionFactory))
    }
}

pub struct NoiseThresholdDeserializer;

impl Deserializer<ConditionFactories> for NoiseThresholdDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        let (noise_name, noise_parameters) = resolver.resolve_field(json, "noise")?;
        
        Ok(
            ConditionFactories::NoiseThreshold(
                NoiseThresholdConditionFactory {
                    noise_name,
                    noise_parameters,
                    min_threshold: resolver.resolve_field(json, "min_threshold")?,
                    max_threshold: resolver.resolve_field(json, "max_threshold")?,
                }
            )
        )
    }
}

pub struct NotDeserializer;

impl Deserializer<ConditionFactories> for NotDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        let Value::Object(map) = json else {
            return Err(anyhow::anyhow!("Expected Object, but given: {:?}", json));
        };

        if !map.contains_key("invert") {
            return Err(anyhow::anyhow!("No \"result_state\" field present"));
        }
        let invert = map.get("invert").unwrap();

        Ok(
            ConditionFactories::Not(
                Box::new(
                    NotConditionFactory(resolver.resolve(invert)?)
                )
            )
        )
    }
}

pub struct SteepMaterialDeserializer;

impl Deserializer<ConditionFactories> for SteepMaterialDeserializer {
    fn deserialize(&self, _: &Value, _: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(ConditionFactories::SteepMaterial(SteepMaterialConditionFactory))
    }
}

pub struct StoneDepthDeserializer;

impl Deserializer<ConditionFactories> for StoneDepthDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(
            ConditionFactories::StoneDepth(
                StoneDepthConditionFactory {
                    surface_type: resolver.resolve_field(json, "surface_type")?,
                    add_surface_depth: resolver.resolve_field(json, "add_surface_depth")?,
                    offset: resolver.resolve_field(json, "offset")?,
                    secondary_depth_range: resolver.resolve_field(json, "secondary_depth_range")?,
                }
            )
        )
    }
}

pub struct TemperatureDeserializer;

impl Deserializer<ConditionFactories> for TemperatureDeserializer {
    fn deserialize(&self, _: &Value, _: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(ConditionFactories::Temperature(TemperatureConditionFactory))
    }
}

pub struct VerticalGradientDeserializer;

impl Deserializer<ConditionFactories> for VerticalGradientDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(
            ConditionFactories::VerticalGradient(
                VerticalGradientConditionFactory {
                    random_name: resolver.resolve_field(json, "random_name")?,
                    true_at_and_below: resolver.resolve_field(json, "true_at_and_below")?,
                    false_at_and_above: resolver.resolve_field(json, "false_at_and_above")?,
                }
            )
        )
    }
}

pub struct WaterDeserializer;

impl Deserializer<ConditionFactories> for WaterDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(
            ConditionFactories::Water(
                WaterConditionFactory {
                    offset: resolver.resolve_field(json, "offset")?,
                    surface_depth_multiplier: resolver.resolve_field(json, "surface_depth_multiplier")?,
                    add_stone_depth: resolver.resolve_field(json, "add_stone_depth")?,
                }
            )
        )
    }
}

pub struct YAboveDeserializer;

impl Deserializer<ConditionFactories> for YAboveDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<ConditionFactories>) -> anyhow::Result<ConditionFactories> {
        Ok(
            ConditionFactories::YAbove(
                YAboveConditionFactory {
                    anchor: resolver.resolve_field(json, "anchor")?,
                    surface_depth_multiplier: resolver.resolve_field(json, "surface_depth_multiplier")?,
                    add_stone_depth: resolver.resolve_field(json, "add_stone_depth")?,
                }
            )
        )
    }
}
