use crate::noise::density::density::DensityFunctions;
use crate::noise::density::unary::{Abs, Cube, HalfNegative, QuarterNegative, Square, Squeeze};
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use serde_json::Value;

pub struct AbsDeserializer;

impl Deserializer<DensityFunctions> for AbsDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Abs(
            Box::new(
                Abs::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct SqueezeDeserializer;

impl Deserializer<DensityFunctions> for SqueezeDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Squeeze(
            Box::new(
                Squeeze::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct QuarterNegativeDeserializer;

impl Deserializer<DensityFunctions> for QuarterNegativeDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::QuarterNegative(
            Box::new(
                QuarterNegative::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct HalfNegativeDeserializer;

impl Deserializer<DensityFunctions> for HalfNegativeDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::HalfNegative(
            Box::new(
                HalfNegative::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct SquareDeserializer;

impl Deserializer<DensityFunctions> for SquareDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Square(
            Box::new(
                Square::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}

pub struct CubeDeserializer;

impl Deserializer<DensityFunctions> for CubeDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Cube(
            Box::new(
                Cube::new(
                    resolver.resolve_field(json, "argument")?
                )
            )
        ))
    }
}
