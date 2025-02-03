use crate::noise::density::misc::RarityValue;
use crate::noise::json::resolvable::Resolvable;
use crate::noise::json::Resolver;
use anyhow::anyhow;
use serde_json::Value;

impl<T> Resolvable<T> for RarityValue {
    fn resolve(val: &Value, _: &Resolver<T>) -> anyhow::Result<Self> where Self: Sized {
        let Value::String(val) = val else {
            return Err(anyhow!("Expected String, but given: {val}"))
        };

        match val.as_ref() {
            "type_1" => Ok(RarityValue::Type1),
            "type_2" => Ok(RarityValue::Type2),
            val => Err(anyhow!("Expected rarity value to be either \"type_1\" or \"type_2\", but \"{val}\" given"))
        }
    }
}
