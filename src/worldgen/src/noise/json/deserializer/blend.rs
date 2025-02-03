use crate::noise::density::blend::{BlendAlpha, BlendOffset};
use crate::noise::density::density::DensityFunctions;
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use serde_json::Value;

pub struct BlendAlphaDeserializer;

impl Deserializer<DensityFunctions> for BlendAlphaDeserializer {
    fn deserialize(&self, _: &Value, _: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::BlendAlpha(BlendAlpha))
    }
}

pub struct BlendOffsetDeserializer;

impl Deserializer<DensityFunctions> for BlendOffsetDeserializer {
    fn deserialize(&self, _: &Value, _: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::BlendOffset(BlendOffset))
    }
}
