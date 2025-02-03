use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;
use std::sync::Arc;

use crate::block::state::BlockState;
use crate::chunk::biome::Biome;
use crate::chunk::palette::local::LocalId;
use bimap::{BiHashMap, Overwritten};
use gxhash::GxBuildHasher;
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use spherix_proto::io::{Error, VarInt, Writable};

/// Represents object ID within [`GlobalPalette`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlobalId(pub u16);

impl From<LocalId> for GlobalId {
    fn from(value: LocalId) -> Self {
        GlobalId(value.0)
    }
}

impl Writable for GlobalId {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        VarInt(self.0 as i32).write(buf)
    }
}

/// Type, that can be associated with value stored in [`GlobalPalette`]. It is used
/// as an alternative value to search within [`GlobalPalette`].
pub trait AsAltPaletteIndex {
    type Index: Eq + Hash;

    fn as_index(&self) -> Self::Index;

    fn as_index_ref(&self) -> &Self::Index;
}

pub struct GlobalPalette<T>
where
    T: Eq + Hash + AsAltPaletteIndex
{
    bits_per_entry: u8,
    map: BiHashMap<GlobalId, Arc<T>, GxBuildHasher, GxBuildHasher>,
    alt_index_link: HashMap<T::Index, Vec<GlobalId>, GxBuildHasher>
}

pub type BlockGlobalPalette = GlobalPalette<BlockState>;
pub type BiomeGlobalPalette = GlobalPalette<Biome>;

impl <T> GlobalPalette<T>
where
    T: Eq + Hash + AsAltPaletteIndex,
{
    pub fn new(bits_per_entry: u8) -> Self {
        Self {
            bits_per_entry,
            map: BiHashMap::with_hashers(GxBuildHasher::default(), GxBuildHasher::default()),
            alt_index_link: HashMap::with_hasher(GxBuildHasher::default())
        }
    }

    pub fn insert(&mut self, id: GlobalId, obj: T) {
        let arc = Arc::new(obj);
        let insert_result = self.map.insert(id, arc.clone());

        match insert_result {
            Overwritten::Neither => {}
            _ => panic!("BiMap replacement is a marker of a bug! Probably, something went wrong.")
        }

        let alt = arc.as_index_ref();
        let link_name = self.alt_index_link.get_mut(alt);

        if link_name.is_none() {
            self.alt_index_link.insert(arc.as_index(), vec![id]);
        } else {
            link_name.unwrap().push(id);
        }
    }

    pub fn get_obj_by_id(&self, id: GlobalId) -> Option<Arc<T>> {
        self.map.get_by_left(&id).map(|obj| obj.clone())
    }

    pub fn get_default_obj_by_index(&self, index: &T::Index) -> Option<Arc<T>> {
        let ids = self.alt_index_link.get(index)?;

        let default_id = ids[0];

        Some(self.map.get_by_left(&default_id)?.clone())
    }

    pub fn get_objs_by_index(&self, index: &T::Index) -> Option<Vec<&Arc<T>>> {
        let ids = self.alt_index_link.get(index)?;

        Some(ids.iter().map(|id| self.map.get_by_left(id).expect(&id.0.to_string())).collect())
    }

    pub fn get_id_by_obj(&self, state: &Arc<T>) -> Option<GlobalId> {
        self.map.get_by_right(state).map(|id| *id)
    }

    pub fn all(&self) -> Vec<Arc<T>> {
        let mut v = Vec::new();

        for r in self.map.right_values() {
            v.push(r.clone())
        }

        v
    }

    pub fn bits_per_entry(&self) -> u8 {
        self.bits_per_entry
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }
}

pub fn create_block_global_palette_from_json(json: serde_json::Value) -> BlockGlobalPalette {
    let root = json.as_object().unwrap();
    let mut palette: BlockGlobalPalette = GlobalPalette::new(15);

    let entries: Vec<(u16, BlockState)> = root
        .iter()
        .collect::<Vec<(&String, &serde_json::Value)>>()
        .par_iter()
        .flat_map(|(name, obj)| {
            let mut res = Vec::new();

            let obj = obj.as_object().unwrap();

            let props_possible_values = obj.get("properties");
            let mut props_possible_values_val= &serde_json::Value::Object(serde_json::Map::new());
            if props_possible_values.is_some() {
                props_possible_values_val = props_possible_values.unwrap()
            }

            let states = obj.get("states")
                .expect(&format!("Block with name \"{}\" has no \"states\" key", name));
            let states = states.as_array().unwrap();

            for state in states {
                let map = state.as_object().unwrap();
                let default = map.get("default");
                let mut default_val = false;

                if default.is_some() {
                    default_val = default.unwrap().as_bool().unwrap();
                }

                let (id, block) = BlockState::from_json(
                    String::from(*name),
                    default_val,
                    props_possible_values_val,
                    state
                ).unwrap();

                res.push((id, block));
            }

            res
        })
        .collect();

    for (id, state) in entries {
        palette.insert(GlobalId(id), state);
    }

    palette
}

pub fn create_biome_global_palette_from_json(json: &serde_json::Value) -> BiomeGlobalPalette {
    let root = json.as_array().unwrap();
    let mut palette: BiomeGlobalPalette = GlobalPalette::new(6);

    let entries: Vec<(u16, Biome)> = root
        .iter()
        .map(|val| {
            (
                val.get("id").unwrap().as_u64().unwrap() as u16,
                Biome::from(val.to_owned())
            )
        })
        .collect();

    for (id, state) in entries {
        palette.insert(GlobalId(id), state);
    }

    palette
}
