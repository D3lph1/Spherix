use std::io::{BufRead, Cursor, Read, Write};

use anyhow::anyhow;
use nbt::de::Decoder;
use nbt::i32_array;
use nbt::ser::Encoder;
use rand::{random, Rng};
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use spherix_world::io::Compression;

macro_rules! default {
    ($name:ident, $ty:ty, $expr:expr) => {
        fn $name() -> $ty {
            $expr
        }
    };
}

///
/// https://minecraft.fandom.com/wiki/Player.dat_format
///
#[derive(Serialize, Deserialize, Debug)]
pub struct Properties {
    #[serde(rename = "HurtByTimestamp", default = "default_hurt_by_timestamp")]
    pub hurt_by_timestamp: i32,
    #[serde(rename = "SleepTimer", default = "default_sleep_timer")]
    pub sleep_timer: i32,
    #[serde(rename = "Invulnerable", default = "default_invulnerable")]
    pub invulnerable: bool,
    #[serde(rename = "FallFlying", default = "default_fall_flying")]
    pub fall_flying: bool,
    #[serde(rename = "PortalCooldown", default = "default_portal_cooldown")]
    pub portal_cooldown: i32,
    #[serde(rename = "AbsorptionAmount", default = "default_absorption_amount")]
    pub absorption_amount: f32,
    #[serde(default)]
    pub abilities: Abilities,
    #[serde(rename = "FallDistance", default = "default_fall_distance")]
    pub fall_distance: f32,
    #[serde(rename = "recipeBook", default)]
    pub recipe_book: RecipeBook,
    #[serde(rename = "DeathTime", default = "default_death_time")]
    pub death_time: i16,
    #[serde(rename = "XpSeed", default = "default_xp_seed")]
    pub xp_seed: i32,
    #[serde(rename = "XpTotal", default = "default_xp_total")]
    pub xp_total: i32,
    #[serde(rename = "UUID", default)]
    pub uuid: Uuid,
    #[serde(rename = "playerGameType", default = "default_player_game_type")]
    pub player_game_type: i32,
    #[serde(rename = "seenCredits", default = "default_seen_credits")]
    pub seen_credits: bool,
    #[serde(rename = "Motion", default = "default_motion")]
    pub motion: Vec<f64>,
    #[serde(rename = "Health", default = "default_health")]
    pub health: f32,
    #[serde(rename = "foodSaturationLevel", default = "default_food_saturation_level")]
    pub food_saturation_level: f32,
    #[serde(rename = "Air", default = "default_air")]
    pub air: i16,
    #[serde(rename = "OnGround", default = "default_on_ground")]
    pub on_ground: bool,
    #[serde(rename = "Dimension", default = "default_dimension")]
    pub dimension: String,
    #[serde(rename = "Rotation", default = "default_rotation")]
    pub rotation: Vec<f32>,
    #[serde(rename = "XpLevel", default = "default_xp_level")]
    pub xp_level: i32,
    #[serde(default)]
    pub warden_spawn_tracker: WardenSpawnTracker,
    #[serde(rename = "Score", default = "default_score")]
    pub score: i32,
    #[serde(rename = "Pos", default = "default_pos")]
    pub pos: Vec<f64>,
    #[serde(rename = "Fire", default = "default_fire")]
    pub fire: i16,
    #[serde(rename = "XpP", default = "default_xp_p")]
    pub xp_p: f32,
    #[serde(rename = "EnderItems", default = "default_ender_items")]
    pub ender_items: Vec<Item>,
    #[serde(rename = "DataVersion", default = "default_data_version")]
    pub data_version: i32,
    #[serde(rename = "foodLevel", default = "default_food_level")]
    pub food_level: i32,
    #[serde(rename = "foodExhaustionLevel", default = "default_food_exhaustion_level")]
    pub food_exhaustion_level: f32,
    #[serde(rename = "HurtTime", default = "default_hurt_time")]
    pub hurt_time: i16,
    #[serde(rename = "SelectedItemSlot", default = "default_selected_item_slot")]
    pub selected_item_slot: i32,
    #[serde(rename = "Inventory", default = "default_inventory")]
    pub inventory: Vec<Item>,
    #[serde(rename = "foodTickTimer", default = "default_food_tick_timer")]
    pub food_tick_timer: i32,
}

default!(default_hurt_by_timestamp, i32, 0);
default!(default_sleep_timer, i32, 0);
default!(default_invulnerable, bool, false);
default!(default_fall_flying, bool, false);
default!(default_portal_cooldown, i32, 0);
default!(default_absorption_amount, f32, 0.0);
default!(default_fall_distance, f32, 0.0);
default!(default_death_time, i16, 0);

fn default_xp_seed() -> i32 {
    random()
}

default!(default_xp_total, i32, 0);
default!(default_player_game_type, i32, 0);
default!(default_seen_credits, bool, false);
default!(default_motion, Vec<f64>, vec![0.0, 0.0, 0.0]);
default!(default_health, f32, 20.0);
default!(default_food_saturation_level, f32, 5.0);
default!(default_air, i16, 300);
default!(default_on_ground, bool, true);
default!(default_dimension, String, String::from("minecraft:overworld"));
default!(default_rotation, Vec<f32>, vec![0.0, 0.0]);
default!(default_xp_level, i32, 0);
default!(default_score, i32, 0);
default!(default_pos, Vec<f64>, vec![0.0, 0.0, 0.0]);
default!(default_fire, i16, -20);
default!(default_xp_p, f32, 0.0);
default!(default_ender_items, Vec<Item>, vec![]);
default!(default_data_version, i32, 0);
default!(default_food_level, i32, 20);
default!(default_food_exhaustion_level, f32, 0.0);
default!(default_hurt_time, i16, 0);
default!(default_selected_item_slot, i32, 0);
default!(default_inventory, Vec<Item>, vec![]);
default!(default_food_tick_timer, i32, 0);

impl Properties {
    pub fn read<R: BufRead>(buf_read: &mut R, compression: Option<Compression>) -> anyhow::Result<Self> {
        let result = match compression {
            None => Self::deserialize(&mut Decoder::new(buf_read)),
            Some(compression) => {
                let buf = compression.decode(buf_read);
                Self::deserialize(&mut Decoder::new(Cursor::new(buf)))
            }
        };

        Ok(result?)
    }

    pub fn write<W: Write>(&self, buf: &mut W, compression: Option<Compression>) -> anyhow::Result<()> {
        let result = match compression {
            None => self.serialize(&mut Encoder::new(buf, None))
                .map_err(|e| anyhow!(e)),
            Some(compression) => self.serialize(
                &mut Encoder::new(compression.wrap_encoder(buf, Default::default()), None)
            )
                .map_err(|e| anyhow!(e))
        };

        Ok(result?)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Abilities {
    pub invulnerable: bool,
    pub mayfly: bool,
    pub instabuild: bool,
    #[serde(rename = "walkSpeed")]
    pub walk_speed: f32,
    #[serde(rename = "mayBuild")]
    pub may_build: bool,
    pub flying: bool,
    #[serde(rename = "flySpeed")]
    pub fly_speed: f32,
}

impl Default for Abilities {
    fn default() -> Self {
        Self {
            invulnerable: false,
            mayfly: false,
            instabuild: false,
            walk_speed: 0.1,
            may_build: true,
            flying: false,
            fly_speed: 0.05,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Uuid(uuid::Uuid);

impl Uuid {
    #[inline]
    fn as_array(&self) -> [i32; 4] {
        let bytes = self.0.as_bytes();

        [
            i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]),
            i32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]),
            i32::from_be_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]),
            i32::from_be_bytes([bytes[12], bytes[13], bytes[14], bytes[15]]),
        ]
    }
}

impl TryFrom<[i32; 4]> for Uuid {
    type Error = anyhow::Error;

    fn try_from(value: [i32; 4]) -> Result<Self, Self::Error> {
        let a = value[0].to_be_bytes() as [u8; 4];
        let b = value[1].to_be_bytes() as [u8; 4];
        let c = value[2].to_be_bytes() as [u8; 4];
        let d = value[3].to_be_bytes() as [u8; 4];

        let bytes = [a, b, c, d].concat();

        Ok(Self(uuid::Uuid::from_bytes(bytes.try_into().unwrap())))
    }
}

impl From<uuid::Uuid> for Uuid {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl Into<uuid::Uuid> for Uuid {
    fn into(self) -> uuid::Uuid {
        self.0
    }
}

impl Default for Uuid {
    fn default() -> Self {
        uuid::Uuid::new_v4().into()
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        i32_array(self.as_array(), serializer)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        Ok(Self::try_from(<[i32; 4] as Deserialize>::deserialize(deserializer)?).unwrap())
    }
}

///
/// https://minecraft.fandom.com/wiki/Recipe_book#Data_values
///
#[derive(Serialize, Deserialize, Debug)]
pub struct RecipeBook {
    recipes: Vec<String>,
    #[serde(rename = "toBeDisplayed")]
    to_be_displayed: Vec<String>,
    #[serde(rename = "isBlastingFurnaceFilteringCraftable")]
    is_blasting_furnace_filtering_craftable: bool,
    #[serde(rename = "isSmokerGuiOpen")]
    is_smoker_gui_open: bool,
    #[serde(rename = "isFilteringCraftable")]
    is_filtering_craftable: bool,
    #[serde(rename = "isFurnaceGuiOpen")]
    is_furnace_gui_open: bool,
    #[serde(rename = "isGuiOpen")]
    is_gui_open: bool,
    #[serde(rename = "isFurnaceFilteringCraftable")]
    is_furnace_filtering_craftable: bool,
    #[serde(rename = "isBlastingFurnaceGuiOpen")]
    is_blasting_furnace_gui_open: bool,
    #[serde(rename = "isSmokerFilteringCraftable")]
    is_smoker_filtering_craftable: bool,
}

impl Default for RecipeBook {
    fn default() -> Self {
        Self {
            recipes: vec![],
            to_be_displayed: vec![],
            is_blasting_furnace_filtering_craftable: false,
            is_smoker_gui_open: false,
            is_filtering_craftable: false,
            is_furnace_gui_open: false,
            is_gui_open: false,
            is_furnace_filtering_craftable: false,
            is_blasting_furnace_gui_open: false,
            is_smoker_filtering_craftable: false,
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WardenSpawnTracker {
    warning_level: i32,
    ticks_since_last_warning: i32,
    cooldown_ticks: i32,
}

impl Default for WardenSpawnTracker {
    fn default() -> Self {
        Self {
            warning_level: 0,
            ticks_since_last_warning: 0,
            cooldown_ticks: 0,
        }
    }
}

///
/// https://minecraft.fandom.com/wiki/Player.dat_format#Item_structure
///
#[derive(Serialize, Deserialize, Debug)]
pub struct Item {
    #[serde(rename = "Count")]
    count: i8,
    #[serde(rename = "Slot")]
    slot: i8,
    id: String,
    // tag
}

#[cfg(test)]
mod tests {
    use crate::world::player::properties::Uuid;

    #[test]
    fn uuid() {
        let u1 = Uuid::default();
        let u2: uuid::Uuid = u1.clone().into();
        let u3: Uuid = u2.into();

        assert_eq!(u1, u3);
    }
}
