pub mod climate;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::chunk::biome::climate::Climate;
use crate::chunk::palette::global::AsAltPaletteIndex;
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;

#[derive(Deserialize)]
pub struct Biome {
    id: u16,
    name: String,
    #[serde(rename = "element")]
    climate: Climate
}

// impl Serialize for Biome {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         let mut state = serializer.serialize_map(Some(5))?;
//         state.serialize_entry("id", &(self.id as i16))?;
//         state.serialize_entry("name", &self.name)?;
//         state.serialize_entry("element", &self.element)?;
//
//         state.end()
//     }
// }

impl PartialEq for Biome {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Eq for Biome {}

impl Hash for Biome {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl AsAltPaletteIndex for Biome {
    type Index = String;

    fn as_index(&self) -> Self::Index {
        self.name.clone()
    }

    fn as_index_ref(&self) -> &Self::Index {
        &self.name
    }
}

impl Biome {
    pub fn new(id: u16, name: String, climate: Climate) -> Self {
        Self {
            id,
            name,
            climate
        }
    }

    pub fn name_ref(&self) -> &str {
        &self.name
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn climate(&self) -> &Climate {
        &self.climate
    }

    pub fn to_nbt(&self) -> nbt::Value {
        nbt::Value::Compound(HashMap::from([
            ("id".to_owned(), nbt::Value::Short(self.id as i16)),
            ("name".to_owned(), nbt::Value::String(self.name.clone())),
            ("element".to_owned(), self.climate.to_nbt())
        ]))
    }

    pub fn convert_array_to_nbt(biomes: Vec<Arc<Biome>>) -> nbt::Value {
        nbt::Value::List(biomes.iter().map(|b| b.to_nbt()).collect())
    }
}

impl From<Value> for Biome {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).unwrap()
    }
}
