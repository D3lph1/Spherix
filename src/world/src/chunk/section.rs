use crate::block::state::BlockState;
use crate::chunk::biome::Biome;
use crate::chunk::palette::container::{create_biome_paletted_container_from_nbt, create_block_paletted_container_from_nbt, PalettedContainer};
use crate::chunk::palette::{BiomeGlobalPalette, BlockGlobalPalette};
use crate::chunk::vector::Vector3BlockSection;
use spherix_math::vector::Vector3u;
use spherix_proto::io::{Short, Writable};
use spherix_util::nbt::NbtExt;
use std::sync::Arc;

/// Chunk section as described [`here`].
///
/// [`here`]: https://wiki.vg/Chunk_Format#Chunk_Section_structure
pub struct ChunkSection {
    /// Vertical-stacked index of this section. 0 (corresponds to -4) - for the
    /// lowest one, and 23 (corresponds to 19) - for the highest one.
    idx: i8,
    block_global_palette: Arc<BlockGlobalPalette>,
    pub biome_global_palette: Arc<BiomeGlobalPalette>,
    pub blocks: PalettedContainer,
    pub biomes: PalettedContainer,
    pub non_empty_block_count: usize,
    pub block_light: Option<[u8; 2048]>,
    pub sky_light: Option<[u8; 2048]>
}

impl ChunkSection {
    pub fn new(
        idx: i8,
        block_global_palette: Arc<BlockGlobalPalette>,
        biome_global_palette: Arc<BiomeGlobalPalette>,
        blocks: PalettedContainer,
        biomes: PalettedContainer,
        block_light: Option<[u8; 2048]>,
        sky_light: Option<[u8; 2048]>
    ) -> Self {
        Self{
            idx,
            block_global_palette,
            biome_global_palette,
            blocks,
            biomes,
            non_empty_block_count: 0,
            block_light,
            sky_light
        }
    }

    /// https://minecraft.fandom.com/wiki/Chunk_format#NBT_structure
    pub fn from_nbt(nbt: &nbt::Value, blocks_global_palette: Arc<BlockGlobalPalette>, biomes_global_palette: Arc<BiomeGlobalPalette>) -> Self {
        let section = nbt.as_compound();

        let y = match section.get("Y").unwrap() {
            nbt::Value::Byte(x) => *x,
            nbt::Value::Short(x) => *x as i8,
            nbt::Value::Int(x) => *x as i8,
            nbt::Value::Long(x) => *x as i8,
            _ => panic!()
        };

        let blocks = create_block_paletted_container_from_nbt(nbt, blocks_global_palette.as_ref());
        let biomes = create_biome_paletted_container_from_nbt(nbt, biomes_global_palette.as_ref());

        let block_light_packed;
        let block_light = section.get("BlockLight");
        if block_light.is_some() {
            block_light_packed = Some(block_light.unwrap().as_byte_array().iter().map(|x| *x as u8).collect::<Vec<u8>>().try_into().unwrap());
        } else {
            block_light_packed = None
        }

        let sky_light_packed;
        let sky_light = section.get("SkyLight");
        if sky_light.is_some() {
            sky_light_packed = Some(sky_light.unwrap().as_byte_array().iter().map(|x| *x as u8).collect::<Vec<u8>>().try_into().unwrap());
        } else {
            sky_light_packed = None
        }

        let mut chunk = Self::new(y, blocks_global_palette, biomes_global_palette, blocks, biomes, block_light_packed, sky_light_packed);

        chunk
    }

    pub fn to_packet_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        (self.non_empty_block_count as Short).write(&mut buf).unwrap();

        self.blocks.write(&mut buf).unwrap();
        self.biomes.write(&mut buf).unwrap();

        buf
    }

    pub fn idx(&self) -> i8 {
        self.idx
    }

    /// Input position vector is a vector of chunk-relative coordinates,
    /// not absolute ones!
    pub fn block_state(&self, pos: Vector3BlockSection) -> Option<Arc<BlockState>> {
        let global_id = self.blocks.get(pos.into())?;

        self.block_global_palette.get_obj_by_id(global_id)
    }

    pub fn set_block_state(&mut self, pos: Vector3BlockSection, state: Arc<BlockState>) {
        let global_id = self.block_global_palette.get_id_by_obj(&state).unwrap();

        let previous_global_id = self.blocks.get_and_set(pos.into(), global_id);
        if previous_global_id.is_some() {
            let previous_block_state = self
                .block_global_palette
                .get_obj_by_id(previous_global_id.unwrap())
                .unwrap();

            if !previous_block_state.block().properties.is_air {
                self.non_empty_block_count -= 1;
            }
        }

        if !state.block().properties.is_air {
            self.non_empty_block_count += 1;
        }
    }
    
    pub fn biome(&self, pos: Vector3u) -> Option<Arc<Biome>> {
        let global_id = self.biomes.get(pos)?;

        self.biome_global_palette.get_obj_by_id(global_id)
    }
    
    pub fn set_biome(&mut self, pos: Vector3u, biome: Arc<Biome>) {
        let global_id = self.biome_global_palette.get_id_by_obj(&biome).unwrap();

        self.biomes.set(pos, global_id);
    }
}

impl AsRef<ChunkSection> for ChunkSection {
    fn as_ref(&self) -> &ChunkSection {
        self
    }
}

impl AsMut<ChunkSection> for ChunkSection {
    fn as_mut(&mut self) -> &mut ChunkSection {
        self
    }
}
