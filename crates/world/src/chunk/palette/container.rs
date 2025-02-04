use crate::block::block::Block;
use crate::block::packed::PackedArray;
use crate::block::state::BlockState;
use crate::chunk::biome::Biome;
use crate::chunk::palette::global::{GlobalId, GlobalPalette};
use crate::chunk::palette::local::{create_biome_palette_from_nbt, create_block_palette_from_nbt, LocalId, LocalPalette, LocalPalettes, PutStatus, SingleValuedLocalPalette};
use crate::chunk::palette::resize::LocalPaletteResizer;
use spherix_math::vector::Vector3u;
use spherix_proto::io::{Error, Writable};
use spherix_util::nbt::NbtExt;
use std::io::Write;

pub struct PalettedContainer {
    pub palette: LocalPalettes,
    pub data: PackedArray,
    resizer: LocalPaletteResizer,
    /// How many bits required to represent the largest edge coordinate in
    /// palette. For example, to represent 16 value in the chunk it is
    /// required 4 (2^4 = 16) bits.
    size_bits: u8,
}

impl Writable for PalettedContainer {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let mut written = self.palette.write(buf)?;
        written += self.data.entries().write(buf)?;

        Ok(written)
    }
}

impl PalettedContainer {
    pub fn new(palette: LocalPalettes, data: PackedArray, resizer: LocalPaletteResizer, size_bits: u8) -> Self {
        Self {
            palette,
            data,
            resizer,
            size_bits,
        }
    }

    pub fn get(&self, v: Vector3u) -> Option<GlobalId> {
        self.get_by_index(self.index(v))
    }

    pub fn set(&mut self, v: Vector3u, global: GlobalId) {
        self.set_by_index(self.index(v), global);
    }
    
    pub fn get_and_set(&mut self, v: Vector3u, global: GlobalId) -> Option<GlobalId> {
        let index = self.index(v);
        let previous = self.get_by_index(index);
        self.set_by_index(index, global);
        
        previous
    }

    #[inline]
    fn set_by_index(&mut self, index: usize, global: GlobalId) {
        let local = self.palette.local_by_global(global);
        if local.is_some() {
            self.data.set(index, local.unwrap().0);

            return;
        }

        match self.palette.put(global) {
            PutStatus::Stored { local } => {
                self.data.set(index, local.0);
            }
            PutStatus::NeedResize { bits } => {
                let (mut palette, mut data) = self.resizer.resize(bits, &self.palette, &self.data);

                let local = match palette.put(global) {
                    PutStatus::Stored { local } => local,
                    PutStatus::NeedResize { .. } => unreachable!()
                };

                data.set(index, local.0);

                self.palette = palette;
                self.data = data;
            }
        };
    }

    #[inline]
    fn get_by_index(&self, index: usize) -> Option<GlobalId> {
        self.palette.global_by_local(LocalId(self.data.get(index)))
    }

    #[inline]
    fn index(&self, v: Vector3u) -> usize {
        ((v.y << self.size_bits | v.z) << self.size_bits | v.x) as usize
    }
}

macro_rules! from_nbt_mixed {
    ($nbt:ident, $palette:ident, $fit_size:ident, $total:literal, $resizer:ident, $size_bits:literal) => {
        {
            let mut vec = Vec::new();

            for value in $nbt {
                vec.push(match value {
                    nbt::Value::Byte(x) => *x as u64,
                    nbt::Value::Short(x) => *x as u64,
                    nbt::Value::Int(x) => *x as u64,
                    nbt::Value::Long(x) => *x as u64,
                    x => panic!("Expected nbt::Value::Byte, nbt::Value::Short, nbt::Value::Int or nbt::Value::Long, {:?} given", x)
                });
            }

            PalettedContainer::new($palette, PackedArray::new(vec, $fit_size as usize, $total), $resizer, $size_bits)
        }
    }
}

macro_rules! from_nbt {
    ($data:ident, $palette:ident, $fit_size:ident, $total:literal, $resizer:ident, $size_bits:literal) => {
        PalettedContainer::new($palette, PackedArray::new($data.iter().map(|x| *x as u64).collect(), $fit_size as usize, $total), $resizer, $size_bits)
    }
}

pub fn create_empty_block_paletted_container(global_palette: &GlobalPalette<BlockState>) -> PalettedContainer {
    let resizer = LocalPaletteResizer::new(4, 15, 9);
    let palette = SingleValuedLocalPalette::new(
        global_palette.get_id_by_obj(
            &global_palette.get_default_obj_by_index(&Block::AIR).unwrap()
        ).unwrap()
    );

    PalettedContainer::new(LocalPalettes::SingleValued(palette), PackedArray::zeros(resizer.min_fit_size as usize, 4096), resizer, 4)
}

pub fn create_block_paletted_container_from_nbt(nbt: &nbt::Value, global_palette: &GlobalPalette<BlockState>) -> PalettedContainer {
    let resizer = LocalPaletteResizer::new(4, 15, 9);

    let nbt = nbt.as_compound();
    let block_states = nbt.get("block_states").unwrap().as_compound();
    let palette = create_block_palette_from_nbt(
        block_states.get("palette").unwrap(),
        &resizer,
        global_palette,
    );

    let fit_size = palette.bits();

    if palette.bits() == 0 {
        PalettedContainer::new(palette, PackedArray::zeros(resizer.min_fit_size as usize, 4096), resizer, 4)
    } else {
        match block_states.get("data").unwrap() {
            nbt::Value::List(data) => from_nbt_mixed!(data, palette, fit_size, 4096, resizer, 4),
            nbt::Value::ByteArray(data) => from_nbt!(data, palette, fit_size, 4096, resizer, 4),
            nbt::Value::IntArray(data) => from_nbt!(data, palette, fit_size, 4096, resizer, 4),
            nbt::Value::LongArray(data) => from_nbt!(data, palette, fit_size, 4096, resizer, 4),
            _ => panic!()
        }
    }
}

pub fn create_empty_biome_paletted_container(global_palette: &GlobalPalette<Biome>) -> PalettedContainer {
    let resizer = LocalPaletteResizer::new(1, 6, 4);
    let palette = SingleValuedLocalPalette::new(
        global_palette.get_id_by_obj(
            &global_palette.get_default_obj_by_index(&"minecraft:forest".to_owned()).unwrap()
        ).unwrap()
    );

    PalettedContainer::new(LocalPalettes::SingleValued(palette), PackedArray::zeros(resizer.min_fit_size as usize, 64), resizer, 2)
}


pub fn create_biome_paletted_container_from_nbt(nbt: &nbt::Value, global_palette: &GlobalPalette<Biome>) -> PalettedContainer {
    let resizer = LocalPaletteResizer::new(1, 6, 4);

    let nbt = nbt.as_compound();
    let block_states = nbt.get("biomes").unwrap().as_compound();
    let palette = create_biome_palette_from_nbt(
        block_states.get("palette").unwrap(),
        &resizer,
        global_palette,
    );

    let fit_size = palette.bits();

    if palette.bits() == 0 {
        PalettedContainer::new(palette, PackedArray::zeros(resizer.min_fit_size as usize, 64), resizer, 2)
    } else {
        match block_states.get("data").unwrap() {
            nbt::Value::List(data) => from_nbt_mixed!(data, palette, fit_size, 64, resizer, 2),
            nbt::Value::ByteArray(data) => from_nbt!(data, palette, fit_size, 64, resizer, 2),
            nbt::Value::IntArray(data) => from_nbt!(data, palette, fit_size, 64, resizer, 2),
            nbt::Value::LongArray(data) => from_nbt!(data, palette, fit_size, 64, resizer, 2),
            _ => panic!()
        }
    }
}
