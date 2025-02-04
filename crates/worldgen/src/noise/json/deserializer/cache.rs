use crate::noise::density::cache::{Cache2D, CacheOnce, FlatCache};
use crate::noise::density::density::{DensityFunctionContext, DensityFunctions};
use crate::noise::density::noise::{NoiseDensityFunction, ShiftedNoise};
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::resolver::Resolver;
use serde_json::Value;

pub struct FlatCacheDeserializer;

impl Deserializer<DensityFunctions> for FlatCacheDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::FlatCache(
            Box::new(
                FlatCache::new(
                    resolver.resolve_field(json, "argument")?,
                    &mut DensityFunctionContext::default(),
                    false,
                )
            )
        ))
    }
}

pub struct Cache2DDeserializer;

impl Deserializer<DensityFunctions> for Cache2DDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Cache2D(
            Box::new(
                Cache2D::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct CacheOnceDeserializer;

impl Deserializer<DensityFunctions> for CacheOnceDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::CacheOnce(
            Box::new(
                CacheOnce::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct NoiseDeserializer;

impl Deserializer<DensityFunctions> for NoiseDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Noise(
            NoiseDensityFunction::new(
                resolver.resolve_field(json,"noise")?,
                resolver.resolve_field(json, "xz_scale")?,
                resolver.resolve_field(json, "y_scale")?,
            )
        ))
    }
}

pub struct ShiftedNoiseDeserializer;

impl Deserializer<DensityFunctions> for ShiftedNoiseDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::ShiftedNoise(
            Box::new(
                ShiftedNoise::new(
                    resolver.resolve_field(json, "noise")?,
                    resolver.resolve_field(json, "shift_x")?,
                    resolver.resolve_field(json, "shift_y")?,
                    resolver.resolve_field(json, "shift_z")?,
                    resolver.resolve_field(json, "xz_scale")?,
                    resolver.resolve_field(json, "y_scale")?,
                )
            )
        ))
    }
}
