use crate::block::block::{Block, BLOCKS};
use crate::block::variant::{Variant, VariantVec};
use crate::chunk::palette::global::AsAltPaletteIndex;
use serde_json::{Map, Value};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

#[derive(Debug)]
pub struct BlockState {
    block: &'static Block,
    default: bool,
    variants: VariantVec
}

impl PartialEq<Self> for BlockState {
    fn eq(&self, other: &Self) -> bool {
        self.block.eq(&other.block) && self.default.eq(&other.default) && self.variants.eq(&other.variants)
    }
}

impl Eq for BlockState {}

impl Hash for BlockState {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.block.hash(state);
        self.default.hash(state);
        self.variants.hash(state);
    }
}

impl AsAltPaletteIndex for BlockState {
    type Index = &'static Block;

    fn as_index(&self) -> Self::Index {
        self.block
    }

    fn as_index_ref(&self) -> &Self::Index {
        &self.block
    }
}

impl BlockState {
    pub fn new(block: &'static Block, default: bool, variants: VariantVec) -> Self {
        Self {
            block,
            default,
            variants
        }
    }

    pub fn from_json(name: String, default: bool, props_possible_values: &Value, state: &Value) -> Option<(u16, Self)> {
        let block = BLOCKS.get(&name).unwrap();
        
        let id = state.get("id").unwrap().as_u64()?;
        let props = state.get("properties");

        let variants;

        // Duplicate calls in order to eliminate extra allocation of Map (in the else clause)
        if props.is_some() {
            variants = Variant::from_json_props(
                props_possible_values,
                props?
            );
        } else {
            variants = Variant::from_json_props(
                props_possible_values,
                // -------------------------| This Map |----------------
                &Value::Object(Map::new())
            );
        }
        
        Some((id as u16, BlockState::new(block, default, variants.into())))
    }

    pub fn block(&self) -> &'static Block {
        self.block
    }
    
    pub fn name(&self) -> &str {
        self.block.name()
    }

    pub fn variants(&self) -> &VariantVec {
        &self.variants
    }

    pub fn variants_match(&self, variants: Vec<Variant>) -> bool {
        if self.variants.len() != variants.len() {
            return false
        }

        for (i, variant) in self.variants.iter().enumerate() {
            if !variant.eq(&variants[i]) {
                return false
            }
        }

        true
    }
}

pub trait VariantPermutations {
    fn get_state_by_variants(&self, variants: VariantVec) -> Option<&Arc<BlockState>>;
}

impl VariantPermutations for Vec<&Arc<BlockState>> {
    fn get_state_by_variants(&self, variants: VariantVec) -> Option<&Arc<BlockState>> {
        for state in self.iter() {
            if state.variants == variants {
                return Some(state)
            }
        }

        None
    }
}
