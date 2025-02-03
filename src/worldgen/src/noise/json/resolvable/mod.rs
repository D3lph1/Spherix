use crate::noise::json::resolver::Resolver;
use crate::rng::Rng;
use serde_json::Value;

pub mod std;
pub mod noise;
pub mod misc;

pub trait Resolvable<T> {
    fn resolve(val: &Value, resolver: &Resolver<T>) -> anyhow::Result<Self> where Self: Sized;
}
