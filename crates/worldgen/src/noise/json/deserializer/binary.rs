use crate::noise::density::binary::{Add, AddConst, Max, Min, Mul, MulConst};
use crate::noise::density::density::DensityFunctions;
use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::resolver::Resolver;
use anyhow::anyhow;
use serde_json::Value;

pub struct AddDeserializer;

impl Deserializer<DensityFunctions> for AddDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        let Value::Object(obj) = json else {
            return Err(anyhow!("Not an object"));
        };

        let argument1 = obj.get("argument1");
        if argument1.is_none() {
            return Err(anyhow!("argument1 key not found"));
        }

        let argument1 = argument1.unwrap();
        if argument1.is_number() {
            return Ok(DensityFunctions::AddConst(
                Box::new(
                    AddConst::new(
                        resolver.resolve_field(json, "argument2")?,
                        argument1.as_f64().unwrap(),
                    )
                )
            ));
        }

        Ok(DensityFunctions::Add(
            Box::new(
                Add::new(
                    resolver.resolve_field(json, "argument1")?,
                    resolver.resolve_field(json, "argument2")?,
                )
            )
        ))
    }
}

pub struct MulDeserializer;

impl Deserializer<DensityFunctions> for MulDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        let Value::Object(obj) = json else {
            return Err(anyhow!("Not an object"));
        };

        let argument1 = obj.get("argument1");
        if argument1.is_none() {
            return Err(anyhow!("argument1 key not found"));
        }

        let argument1 = argument1.unwrap();
        if argument1.is_number() {
            return Ok(DensityFunctions::MulConst(
                Box::new(
                    MulConst::new(
                        resolver.resolve_field(json, "argument2")?,
                        argument1.as_f64().unwrap(),
                    )
                )
            ));
        }

        Ok(DensityFunctions::Mul(
            Box::new(
                Mul::new(
                    resolver.resolve_field(json, "argument1")?,
                    resolver.resolve_field(json, "argument2")?,
                )
            )
        ))
    }
}

pub struct MaxDeserializer;

impl Deserializer<DensityFunctions> for MaxDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Max(
            Box::new(
                Max::new(
                    resolver.resolve_field(json, "argument1")?,
                    resolver.resolve_field(json, "argument2")?,
                )
            )
        ))
    }
}

pub struct MinDeserializer;

impl Deserializer<DensityFunctions> for MinDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        Ok(DensityFunctions::Min(
            Box::new(
                Min::new(
                    resolver.resolve_field(json, "argument1")?,
                    resolver.resolve_field(json, "argument2")?,
                )
            )
        ))
    }
}
