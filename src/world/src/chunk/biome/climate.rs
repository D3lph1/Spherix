use gxhash::GxBuildHasher;
use lru::LruCache;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use thread_local::ThreadLocal;

pub struct Climate {
    downfall: f32,
    temperature: Temperature,
    precipitation: String,
    effects: Effects,
}

impl<'de> Deserialize<'de> for Climate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            Downfall,
            Temperature,
            TemperatureModifier,
            Precipitation,
            Effects,
        }

        struct ClimateVisitor;

        impl<'de> Visitor<'de> for ClimateVisitor {
            type Value = Climate;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Climate")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut downfall = None;
                let mut temperature = None;
                let mut temperature_modifier = None;
                let mut precipitation = None;
                let mut effects = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::Downfall => {
                            downfall = Some(map.next_value()?);
                        }
                        Field::Temperature => {
                            temperature = Some(map.next_value()?);
                        }
                        Field::TemperatureModifier => {
                            temperature_modifier = Some(map.next_value()?);
                        }
                        Field::Precipitation => {
                            precipitation = Some(map.next_value()?);
                        }
                        Field::Effects => {
                            effects = Some(map.next_value()?);
                        }
                    }
                }

                let downfall = downfall.ok_or_else(|| serde::de::Error::missing_field("downfall"))?;
                let temperature = temperature.ok_or_else(|| serde::de::Error::missing_field("temperature"))?;
                let temperature_modifier = temperature_modifier.unwrap_or_else(|| TemperatureModifier::default());
                let precipitation = precipitation.unwrap_or_default();
                let effects = effects.unwrap();

                Ok(Climate {
                    downfall,
                    temperature: Temperature {
                        value: temperature,
                        modifier: temperature_modifier,
                        cache: ThreadLocal::new(),
                    },
                    precipitation,
                    effects,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["downfall", "temperature", "temperature_modifier", "precipitation", "effects"];
        deserializer.deserialize_struct("Climate", FIELDS, ClimateVisitor)
    }
}

// impl Serialize for Climate {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         let mut state = serializer.serialize_map(Some(5))?;
//         state.serialize_entry("downfall", &self.downfall)?;
//         state.serialize_entry("temperature", &self.temperature)?;
//         state.serialize_entry("precipitation", &self.precipitation)?;
//         state.serialize_entry("effects", &self.effects)?;
//         state.serialize_entry("has_precipitation", &i8::from(self.precipitation == "none"))?;
//
//         state.end()
//     }
// }

impl Climate {
    #[inline]
    pub fn temperature(&self) -> &Temperature {
        &self.temperature
    }

    pub fn to_nbt(&self) -> nbt::Value {
        nbt::Value::Compound(HashMap::from([
            ("downfall".to_owned(), nbt::Value::Float(self.downfall.into())),
            ("temperature".to_owned(), nbt::Value::Float(self.temperature.value.into())),
            ("precipitation".to_owned(), nbt::Value::String(self.precipitation.clone())),
            ("effects".to_owned(), self.effects.to_nbt()),
            ("has_precipitation".to_owned(), nbt::Value::Byte(i8::from(self.precipitation == "none")))
        ]))
    }
}

pub type TemperatureCache = ThreadLocal<RefCell<LruCache<i64, f32, GxBuildHasher>>>;

pub struct Temperature {
    pub value: f32,
    pub modifier: TemperatureModifier,
    // Mb extract cache to worldgen?
    pub cache: TemperatureCache
}

#[derive(Deserialize, Clone, Copy)]
pub enum TemperatureModifier {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "frozen")]
    Frozen
}

impl Default for TemperatureModifier {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Deserialize)]
pub struct Effects {
    sky_color: u32,
    foliage_color: Option<u32>,
    water_fog_color: u32,
    water_color: u32,
    grass_color_modifier: Option<String>,
    fog_color: u32,
    music: Option<Music>,
    mood_sound: MoodSound,
}

// impl Serialize for Effects {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         let mut state = serializer.serialize_map(Some(5))?;
//         state.serialize_entry("sky_color", &(self.sky_color as i32))?;
//         state.serialize_entry("water_fog_color", &(self.water_fog_color as i32))?;
//         state.serialize_entry("water_color", &(self.water_color as i32))?;
//         state.serialize_entry("fog_color", &(self.fog_color as i32))?;
//         state.serialize_entry("mood_sound", &self.mood_sound)?;
//
//         if self.foliage_color.is_some() {
//             state.serialize_entry("foliage_color", &(self.foliage_color.unwrap() as i32))?;
//         }
//
//         if self.grass_color_modifier.is_some() {
//             state.serialize_entry("grass_color_modifier", &self.grass_color_modifier.as_ref().unwrap().clone())?;
//         }
//
//         if self.music.is_some() {
//             state.serialize_entry("music", &self.music.as_ref().unwrap())?;
//         }
//
//         state.end()
//     }
// }

impl Effects {
    fn to_nbt(&self) -> nbt::Value {
        let mut v = vec![
            ("sky_color".to_owned(), nbt::Value::Int(self.sky_color as i32)),
            ("water_fog_color".to_owned(), nbt::Value::Int(self.water_fog_color as i32)),
            ("water_color".to_owned(), nbt::Value::Int(self.water_color as i32)),
            ("fog_color".to_owned(), nbt::Value::Int(self.fog_color as i32)),
            ("mood_sound".to_owned(), self.mood_sound.to_nbt())
        ];

        if self.foliage_color.is_some() {
            v.push(("foliage_color".to_owned(), nbt::Value::Int(self.foliage_color.unwrap() as i32)));
        }

        if self.grass_color_modifier.is_some() {
            v.push(("grass_color_modifier".to_owned(), nbt::Value::String(self.grass_color_modifier.as_ref().unwrap().clone())));
        }

        if self.music.is_some() {
            v.push(("music".to_owned(), self.music.as_ref().unwrap().to_nbt()))
        }

        nbt::Value::Compound(HashMap::from_iter(v))
    }
}

#[derive(Deserialize)]
pub struct MoodSound {
    tick_delay: u16,
    offset: f32,
    sound: String,
    block_search_extent: u8,
}

// impl Serialize for MoodSound {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer
//     {
//         let mut state = serializer.serialize_map(Some(4))?;
//         state.serialize_entry("tick_delay", &self.tick_delay)?;
//         state.serialize_entry("offset", &self.offset)?;
//         state.serialize_entry("sound", &self.sound)?;
//         state.serialize_entry("block_search_extent", &(self.block_search_extent as i8))?;
//
//         state.end()
//     }
// }

impl MoodSound {
    fn to_nbt(&self) -> nbt::Value {
        nbt::Value::Compound(HashMap::from([
            ("tick_delay".to_owned(), nbt::Value::Short(self.tick_delay as i16)),
            ("offset".to_owned(), nbt::Value::Float(self.offset.clone().into())),
            ("sound".to_owned(), nbt::Value::String(self.sound.clone())),
            ("block_search_extent".to_owned(), nbt::Value::Byte(self.block_search_extent as i8))
        ]))
    }
}

#[derive(PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Music {
    replace_current_music: u32,
    sound: String,
    min_delay: u32,
    max_delay: u32,
}

impl Music {
    fn to_nbt(&self) -> nbt::Value {
        nbt::Value::Compound(HashMap::from([
            ("replace_current_music".to_owned(), nbt::Value::Int(self.replace_current_music as i32)),
            ("sound".to_owned(), nbt::Value::String(self.sound.clone())),
            ("min_delay".to_owned(), nbt::Value::Int(self.min_delay as i32)),
            ("max_delay".to_owned(), nbt::Value::Int(self.max_delay as i32))
        ]))
    }
}
