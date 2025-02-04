use crate::noise::density::density::DensityFunctions;
use crate::noise::density::maker::{Interpolated, Marker};
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use serde_json::Value;

pub struct InterpolatedDeserializer;

impl Deserializer<DensityFunctions> for InterpolatedDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        let interpolated = Interpolated::new(
            resolver.resolve_field(json, "argument")?,
            16,
            16
        );

        Ok(DensityFunctions::Interpolated(Box::new(interpolated)))
    }
}

pub struct MarkerDeserializer {
    ty: String
}

impl MarkerDeserializer {
    pub fn new(ty: impl Into<String>) -> Self {
        Self {
            ty: ty.into(),
        }
    }
}

impl Deserializer<DensityFunctions> for MarkerDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(
            DensityFunctions::Marker(
                Box::new(
                    Marker::new(
                        self.ty.clone(),
                        resolver.resolve_field(json, "argument")?
                    )
                )
            )
        )
    }
}
