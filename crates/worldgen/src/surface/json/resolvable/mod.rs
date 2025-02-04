use crate::noise::json::resolvable::Resolvable;
use crate::noise::json::Resolver;
use crate::surface::condition::VerticalAnchor;
use serde_json::Value;
use std::hash::Hash;

impl<T> Resolvable<T> for VerticalAnchor {
    fn resolve(val: &Value, _: &Resolver<T>) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        let Value::Object(map) = val else {
            return Err(anyhow::anyhow!("Expected Object, but given: {:?}", val));
        };

        if map.contains_key("above_bottom") {
            Ok(VerticalAnchor::AboveBottom {
                offset: decode_i32(map.get("above_bottom").unwrap())?,
            })
        } else if map.contains_key("below_top") {
            Ok(VerticalAnchor::BelowTop {
                offset: decode_i32(map.get("below_top").unwrap())?,
            })
        } else if map.contains_key("absolute") {
            Ok(VerticalAnchor::Absolute {
                y: decode_i32(map.get("absolute").unwrap())?,
            })
        } else {
            Err(anyhow::anyhow!("Expected VerticalAnchor value to be object with a key equals to either \"above_bottom\", \"below_top\" or \"absolute\", but given: {:?}", val))
        }
    }
}

fn decode_i32(val: &Value) -> anyhow::Result<i32> {
    if !val.is_i64() {
        Err(anyhow::anyhow!("VerticalAnchor must be an integer."))?
    } else {
        Ok(val.as_i64().unwrap() as i32)
    }
}
