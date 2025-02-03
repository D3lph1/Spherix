use crate::noise::density::density::DensityFunction;
use crate::noise::json::resolver::Resolver;
use crate::rng::Rng;
use serde_json::Value;

pub use binary::*;
pub use cache::*;

pub mod cache;
pub mod binary;
pub mod unary;
pub mod maker;
pub mod noise;
pub mod misc;
pub mod blend;
pub mod spline;

pub trait Deserializer<T> {
    fn deserialize(&self, json: &Value, resolver: &Resolver<T>) -> anyhow::Result<T>;
}
