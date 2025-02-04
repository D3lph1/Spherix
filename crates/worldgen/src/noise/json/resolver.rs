use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::resolvable::Resolvable;
use crate::noise::json::value_resolver::ValueResolver;
use anyhow::anyhow;
use serde_json::Value;
use std::cell::Cell;
use std::collections::HashMap;

pub struct Resolver<T> {
    deserializers: HashMap<String, Box<dyn Deserializer<T>>>,
    value_resolver: Box<dyn ValueResolver>,
    pub contextual_name: Cell<Option<String>>
}

impl<T> Resolver<T> {
    pub fn new(
        deserializers: HashMap<String, Box<dyn Deserializer<T>>>,
        value_resolver: Box<dyn ValueResolver>,
    ) -> Self {
        Self {
            deserializers,
            value_resolver,
            contextual_name: Cell::new(None)
        }
    }

    pub fn resolve_value(&self, name: String) -> anyhow::Result<Value> {
        self.value_resolver.resolve(name).map_err(|e| anyhow!(e))
    }

    pub fn resolve(&self, json: &Value) -> anyhow::Result<T> {
        if json.is_string() {
            let s = json.as_str().unwrap().to_owned();
            let value = self.resolve_value(s)?;

            return Ok(self.resolve(&value)?)
        }

        let Value::Object(map) = json else {
            return Err(anyhow!("Expected Object or String, but given: {:?}", json))
        };

        if !map.contains_key("type") {
            return Err(anyhow!("No \"type\" field found"));
        }

        let t = map.get("type").unwrap();

        let Value::String(name) = t else {
            return Err(anyhow!("Expected String, but given: {:?}", t))
        };

        let des = self.deserializers.get(name);
        if des.is_none() {
            return Err(anyhow!("No deserializer for type \"{}\" found", name));
        }

        Ok(des.unwrap().deserialize(&json, self)?)
    }

    pub fn resolve_field<R: Resolvable<T>, S: AsRef<str>>(&self, json: &Value, field: S) -> anyhow::Result<R> {
        match json {
            Value::String(s) => {
                self.contextual_name.replace(Some(s.clone()));
                Ok(self.resolve_field(&self.resolve_value(s.to_owned())?, field)?)
            }
            Value::Object(map) => {
                let val = map.get(field.as_ref());
                if val.is_none() {
                    return Err(anyhow!("Map has no field named {}", field.as_ref()));
                }

                Ok(R::resolve(val.unwrap(), self)?)
            }
            v => Err(anyhow!("Expected String or Object, but given: {:?}", v))
        }
    }

    pub fn deserialize(&self, name: &String) -> anyhow::Result<T> {
        let value = self.resolve_value(name.clone())?;

        if self.deserializers.contains_key(name) {
            Ok(
                self.deserializers
                    .get(name)
                    .unwrap()
                    .deserialize(&value, self)?
            )
        } else {
            let value = self.resolve_value(name.clone()).map_err(|e| {
                anyhow!("Error occurred during resolving value \"{}\": {}", name, e)
            })?;

            self.resolve(&value).map_err(|e| {
                anyhow!("Error occurred during deserialization of value \"{}\": {}", name, e)
            })
        }
    }
}
