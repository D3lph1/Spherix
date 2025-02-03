use crate::block::block::BLOCKS;
use crate::block::state::{BlockState, VariantPermutations};
use crate::block::variant::Variant;
use crate::chunk::biome::Biome;
use crate::chunk::palette::global::{GlobalId, GlobalPalette};
use crate::chunk::palette::resize::LocalPaletteResizer;
use bimap::BiHashMap;
use gxhash::{GxBuildHasher, GxHasher};
use log::warn;
use nalgebra::max;
use spherix_proto::io::{Byte, Error, VarInt, Writable};
use spherix_util::nbt::NbtExt;
use std::collections::HashMap;
use std::hash::Hash;
use std::io::Write;

/// Represents object ID within local palette
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalId(pub u16);

impl From<GlobalId> for LocalId {
    fn from(value: GlobalId) -> Self {
        LocalId(value.0)
    }
}

impl PartialEq<u16> for LocalId {
    fn eq(&self, other: &u16) -> bool {
        self.0 == *other
    }
}

/// Local palette maps [`LocalId`] to [`GlobalId`] and vise versa.
pub trait LocalPalette: Writable {
    fn global_by_local(&self, local: LocalId) -> Option<GlobalId>;
    fn local_by_global(&self, global: GlobalId) -> Option<LocalId>;

    fn put(&mut self, global: GlobalId) -> PutStatus;

    fn len(&self) -> usize;

    fn bits(&self) -> u8;
}

pub enum LocalPalettes {
    SingleValued(SingleValuedLocalPalette),
    HashMap(HashMapLocalPalette),
    Global(GlobalLocalPalette)
}

impl Writable for LocalPalettes {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        match self {
            LocalPalettes::SingleValued(x) => x.write(buf),
            LocalPalettes::HashMap(x) => x.write(buf),
            LocalPalettes::Global(x) => x.write(buf),
        }
    }
}

impl LocalPalette for LocalPalettes {
    fn global_by_local(&self, local: LocalId) -> Option<GlobalId> {
        match self {
            LocalPalettes::SingleValued(x) => x.global_by_local(local),
            LocalPalettes::HashMap(x) => x.global_by_local(local),
            LocalPalettes::Global(x) => x.global_by_local(local),
        }
    }

    fn local_by_global(&self, global: GlobalId) -> Option<LocalId> {
        match self {
            LocalPalettes::SingleValued(x) => x.local_by_global(global),
            LocalPalettes::HashMap(x) => x.local_by_global(global),
            LocalPalettes::Global(x) => x.local_by_global(global),
        }
    }

    fn put(&mut self, global: GlobalId) -> PutStatus {
        match self {
            LocalPalettes::SingleValued(x) => x.put(global),
            LocalPalettes::HashMap(x) => x.put(global),
            LocalPalettes::Global(x) => x.put(global),
        }
    }

    fn len(&self) -> usize {
        match self {
            LocalPalettes::SingleValued(x) => x.len(),
            LocalPalettes::HashMap(x) => x.len(),
            LocalPalettes::Global(x) => x.len()
        }
    }

    fn bits(&self) -> u8 {
        match self {
            LocalPalettes::SingleValued(x) => x.bits(),
            LocalPalettes::HashMap(x) => x.bits(),
            LocalPalettes::Global(x) => x.bits()
        }
    }
}

pub enum PutStatus {
    Stored {
        local: LocalId
    },
    NeedResize {
        bits: u8
    }
}

pub struct SingleValuedLocalPalette(GlobalId);

impl SingleValuedLocalPalette {
    pub fn new(value: GlobalId) -> Self {
        Self(value)
    }
}

impl Writable for SingleValuedLocalPalette
{
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        Ok((0 as Byte).write(buf)? + self.0.write(buf)?)
    }
}

impl LocalPalette for SingleValuedLocalPalette
{
    fn global_by_local(&self, local: LocalId) -> Option<GlobalId> {
        if local == 0 {
            Some(self.0)
        } else {
            None
        }
    }

    fn local_by_global(&self, global: GlobalId) -> Option<LocalId> {
        if global == self.0 {
            Some(LocalId(0))
        } else {
            None
        }
    }

    fn put(&mut self, global: GlobalId) -> PutStatus {
        if global == self.0 {
            PutStatus::Stored {
                local: LocalId(0),
            }
        } else {
            PutStatus::NeedResize {
                bits: 1
            }
        }
    }

    fn len(&self) -> usize {
        1
    }

    fn bits(&self) -> u8 {
        0
    }
}

struct IncrementalHashMap {
    bi: BiHashMap<LocalId, GlobalId, GxBuildHasher, GxBuildHasher>
}

impl IncrementalHashMap
{
    #[inline]
    fn new() -> Self {
        Self {
            bi: BiHashMap::with_hashers(GxBuildHasher::default(), GxBuildHasher::default()),
        }
    }

    #[inline]
    fn with_capacity(cap: usize) -> Self {
        Self {
            bi: BiHashMap::with_capacity_and_hashers(cap, GxBuildHasher::default(), GxBuildHasher::default()),
        }
    }

    #[inline]
    fn global_by_local(&self, local: LocalId) -> Option<GlobalId> {
        self.bi.get_by_left(&local).map(|global| *global)
    }

    #[inline]
    fn local_by_global(&self, global: GlobalId) -> Option<LocalId> {
        self.bi.get_by_right(&global).map(|local| *local)
    }

    #[inline]
    fn put(&mut self, global: GlobalId) -> LocalId {
        let id = LocalId(self.bi.len() as u16);

        match self.bi.insert(id, global) {
            _ => id
        }
    }

    #[inline]
    fn len(&self) -> usize {
        self.bi.len()
    }
}

pub struct HashMapLocalPalette {
    map: IncrementalHashMap,
    bits: u8,
}

impl HashMapLocalPalette
{
    pub fn new(bits: u8) -> Self {
        Self {
            map: IncrementalHashMap::new(),
            bits,
        }
    }

    pub fn with_capacity(bits: u8, cap: usize) -> Self {
        Self {
            map: IncrementalHashMap::with_capacity(cap),
            bits,
        }
    }
}

impl Writable for HashMapLocalPalette
{
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let mut written = self.bits.write(buf)?;
        written += VarInt(self.map.len() as i32).write(buf)?;
        for i in 0..self.map.len() {
            written += self.map.global_by_local(LocalId(i as u16)).unwrap().write(buf)?;
        }

        Ok(written)
    }
}

impl LocalPalette for HashMapLocalPalette
{
    fn global_by_local(&self, local: LocalId) -> Option<GlobalId> {
        self.map.global_by_local(local)
    }

    fn local_by_global(&self, global: GlobalId) -> Option<LocalId> {
        self.map.local_by_global(global)
    }

    fn put(&mut self, global: GlobalId) -> PutStatus {
        if self.map.len() >= 1 << self.bits {
            PutStatus::NeedResize {
                bits: self.bits + 1
            }
        } else {
            let local = self.map.put(global);

            PutStatus::Stored {
                local,
            }
        }
    }

    fn len(&self) -> usize {
        self.map.len()
    }

    fn bits(&self) -> u8 {
        self.bits
    }
}

pub struct GlobalLocalPalette {
    bits: u8
}

impl GlobalLocalPalette {
    pub fn new(bits: u8) -> Self {
        Self {
            bits,
        }
    }
}

impl Writable for GlobalLocalPalette {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        Ok(0)
    }
}

impl LocalPalette for GlobalLocalPalette {
    fn global_by_local(&self, local: LocalId) -> Option<GlobalId> {
        Some(local.into())
    }

    fn local_by_global(&self, global: GlobalId) -> Option<LocalId> {
        Some(global.into())
    }

    fn put(&mut self, global: GlobalId) -> PutStatus {
        PutStatus::Stored {
            local: global.into(),
        }
    }

    fn len(&self) -> usize {
        0 // length is not important for this palette
    }

    fn bits(&self) -> u8 {
        self.bits
    }
}

pub fn create_block_palette_from_nbt(nbt: &nbt::Value, resizer: &LocalPaletteResizer, global_palette: &GlobalPalette<BlockState>) -> LocalPalettes {
    let items = nbt.as_list();
    let bits = (items.len() as f64).log2().ceil() as u8;

    if bits == 0 {
        let item = items[0].as_compound();
        let global = global_id_of_block(item, &global_palette);

        LocalPalettes::SingleValued(SingleValuedLocalPalette::new(global))
    } else if bits < resizer.threshold {
        let bits = max(bits, resizer.min_fit_size);

        // Init with capacity?
        let mut palette = HashMapLocalPalette::new(bits);
        for item in items {
            let item = item.as_compound();
            let global = global_id_of_block(item, &global_palette);
            match palette.put(global) {
                PutStatus::NeedResize { .. } => {
                    warn!("Resizing attempt during palette creation is a marker of bug!")
                },
                _ => {}
            }
        }

        LocalPalettes::HashMap(palette)
    } else {
        LocalPalettes::Global(GlobalLocalPalette::new(resizer.max_fit_size))
    }
}

fn global_id_of_block(item: &HashMap<String, nbt::Value>, global_palette: &GlobalPalette<BlockState>) -> GlobalId {
    let name = item.get("Name").unwrap().as_string();
    let block = BLOCKS.get(name).unwrap();
    let states = global_palette.get_objs_by_index(block).unwrap();
    let mut state = states[0].clone();

    if item.contains_key("Properties") {
        let props = item.get("Properties").unwrap().as_compound();

        let variants = Variant::from_nbt(props, state.variants())
            .expect(&format!("Variant::from_nbt: {:?} {:?} {:?}", name, props, state.variants()));

        state = states.get_state_by_variants(variants).unwrap().clone();
    } else {
        if states.len() != 1 {
            panic!("Illegal state")
        }
    }

    global_palette.get_id_by_obj(&state).unwrap()
}

pub fn create_biome_palette_from_nbt(nbt: &nbt::Value, resizer: &LocalPaletteResizer, global_palette: &GlobalPalette<Biome>) -> LocalPalettes {
    let items = nbt.as_list();
    let bits = (items.len() as f64).log2().ceil() as u8;

    if bits == 0 {
        let item = items[0].as_string();
        let global = global_id_of_biome(item, &global_palette);

        LocalPalettes::SingleValued(SingleValuedLocalPalette::new(global))
    } else if bits < resizer.threshold {
        // Init with capacity?
        let mut palette = HashMapLocalPalette::new(bits);
        for item in items {
            let item = item.as_string();
            let global = global_id_of_biome(item, &global_palette);
            palette.put(global);
        }

        LocalPalettes::HashMap(palette)
    } else {
        LocalPalettes::Global(GlobalLocalPalette::new(resizer.max_fit_size))
    }
}

fn global_id_of_biome(item: &String, global_palette: &GlobalPalette<Biome>) -> GlobalId {
    // todo: get id be name directly
    let biome = global_palette.get_objs_by_index(item).unwrap();
    let biome = biome.first().unwrap();
    global_palette.get_id_by_obj(biome).unwrap()
}
