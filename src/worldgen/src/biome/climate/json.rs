use crate::biome::climate::decimal::stringed_float_to_i64;
use crate::biome::climate::point::ClimatePoint;
use crate::biome::climate::rtree::Rectangle;
use rstar::primitives::GeomWithData;
use rstar::RTree;
use serde::{Deserialize, Deserializer};

/// Due to [`#497`], it is currently not possible
/// to use [`serde_json::value::RawValue`] inside [`untagged enum`]. So I have to write
/// manual deserialization of these variants (see [`deserialize_value`]).
///
/// [`#497`]: https://github.com/serde-rs/json/issues/497
/// [`untagged enum`]: https://serde.rs/enum-representations.html#untagged
pub enum Value {
    Range(i64, i64),
    Const(i64)
}

impl Value {
    fn into_range(self) -> (i64, i64) {
        match self {
            Value::Range(min, max) => (min, max),
            Value::Const(num) => (num, num)
        }
    }
}

/// In this scenario, we needed to read floating-point values from JSON and convert them
/// to scaled integers (i32) without losing precision. The values are stored in JSON as
/// numeric literals (e.g., 3.05 or -1.2). However, during deserialization, serde
/// automatically interprets these numeric literals as f64, which immediately results
/// in the precision loss we want to avoid. We needed to access these values as
/// raw strings to manually parse and scale them for the desired precision.
fn deserialize_value<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    let value: &serde_json::value::RawValue = Deserialize::deserialize(deserializer)?;
    let str = value.get();

    if str.starts_with('[') && str.ends_with(']') {
        let parts: Vec<&str> = str[1..str.len() - 1].split(',').collect();
        if parts.len() == 2 {
            let first = stringed_float_to_i64(parts[0].trim(), 4)
                .map_err(|e| serde::de::Error::custom(e.to_string()))?;
            let second = stringed_float_to_i64(parts[1].trim(), 4)
                .map_err(|e| serde::de::Error::custom(e.to_string()))?;

            Ok(Value::Range(first, second))
        } else {
            Err(serde::de::Error::custom("Expected exactly two elements in range array"))
        }
    } else {
        let single_value = stringed_float_to_i64(str, 4)
            .map_err(|e| serde::de::Error::custom(e.to_string()))?;

        Ok(Value::Const(single_value))
    }
}

#[derive(Deserialize)]
pub struct Parameters {
    #[serde(deserialize_with = "deserialize_value")]
    continentalness: Value,
    #[serde(deserialize_with = "deserialize_value")]
    depth: Value,
    #[serde(deserialize_with = "deserialize_value")]
    erosion: Value,
    #[serde(deserialize_with = "deserialize_value")]
    humidity: Value,
    #[serde(deserialize_with = "deserialize_value")]
    temperature: Value,
    #[serde(deserialize_with = "deserialize_value")]
    weirdness: Value
}

#[derive(Deserialize)]
pub struct Biome {
    biome: String,
    parameters: Parameters
}

#[derive(Deserialize)]
pub struct Biomes {
    biomes: Vec<Biome>
}

pub type BiomeIndex = RTree<GeomWithData<Rectangle<ClimatePoint>, String>>;

pub fn create_biome_index_from_json(s: String) -> anyhow::Result<BiomeIndex> {
    let mut rtree = RTree::new();
    let biomes: Biomes = serde_json::from_str(&s).unwrap();

    for biome in biomes.biomes.into_iter() {
        let parameters = biome.parameters;

        let temperature = parameters.temperature.into_range();
        let humidity = parameters.humidity.into_range();
        let continentalness = parameters.continentalness.into_range();
        let erosion = parameters.erosion.into_range();
        let depth = parameters.depth.into_range();
        let weirdness = parameters.weirdness.into_range();

        let lower = ClimatePoint {
            temperature: temperature.0,
            humidity: humidity.0,
            continentalness: continentalness.0,
            erosion: erosion.0,
            depth: depth.0,
            weirdness: weirdness.0,
        };

        let upper = ClimatePoint {
            temperature: temperature.1,
            humidity: humidity.1,
            continentalness: continentalness.1,
            erosion: erosion.1,
            depth: depth.1,
            weirdness: weirdness.1,
        };

        rtree.insert(
            GeomWithData::new(
                Rectangle::from_corners(lower, upper),
                biome.biome
            )
        );
    }

    Ok(rtree)
}
