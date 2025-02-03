use crate::noise::json::resolvable::Resolvable;
use crate::noise::json::Resolver;
use anyhow::anyhow;
use serde_json::Value;
use std::collections::HashSet;
use std::hash::Hash;

impl<T> Resolvable<T> for i32 {
    fn resolve(val: &Value, _: &Resolver<T>) -> anyhow::Result<Self> {
        let Value::Number(num) = val else {
            return Err(anyhow!("Expected Number, but given: {:?}", val))
        };

        let casted = num.as_i64();
        if casted.is_none() {
            return Err(anyhow!("Expected i64, but given: {:?}", num))
        }
        Ok(casted.unwrap() as i32)
    }
}

impl<T> Resolvable<T> for f64 {
    fn resolve(val: &Value, _: &Resolver<T>) -> anyhow::Result<Self> {
        let Value::Number(num) = val else {
            return Err(anyhow!("Expected Number, but given: {:?}", val))
        };

        let casted = num.as_f64();
        if casted.is_none() {
            return Err(anyhow!("Expected f64, but given: {:?}", num))
        }
        Ok(casted.unwrap())
    }
}

impl<T> Resolvable<T> for f32 {
    fn resolve(val: &Value, resolver: &Resolver<T>) -> anyhow::Result<Self> {
        Ok(f64::resolve(val, resolver)? as f32)
    }
}

impl<T, E> Resolvable<T> for Vec<E> where E: Resolvable<T> {
    fn resolve(val: &Value, resolver: &Resolver<T>) -> anyhow::Result<Self> {
        let Value::Array(vec) = val else {
            return Err(anyhow!("Expected Array, but given: {:?}", val))
        };

        Ok(
            vec
                .clone()
                .iter()
                .map(|x| E::resolve(x, resolver))
                .collect::<anyhow::Result<Vec<E>>>()?
        )
    }
}

impl<T, E> Resolvable<T> for HashSet<E> where E: Resolvable<T> + Eq + Hash {
    fn resolve(val: &Value, resolver: &Resolver<T>) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        let Value::Array(vec) = val else {
            return Err(anyhow::anyhow!("Expected Array, but given: {:?}", val))
        };

        Ok(
            vec
                .clone()
                .iter()
                .map(|x| E::resolve(x, resolver))
                .collect::<anyhow::Result<HashSet<E>>>()?
        )
    }
}

impl<T> Resolvable<T> for String {
    fn resolve(val: &Value, _: &Resolver<T>) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        if val.is_string() {
            Ok(val.as_str().unwrap().to_owned())
        } else {
            Err(anyhow!("Expected String, but given: {:?}", val))
        }
    }
}

impl<T> Resolvable<T> for bool {
    fn resolve(val: &Value, _: &Resolver<T>) -> anyhow::Result<Self>
    where
        Self: Sized
    {
        if val.is_boolean() {
            Ok(val.as_bool().unwrap())
        } else {
            Err(anyhow!("Expected bool, but given: {:?}", val))
        }
    }
}
