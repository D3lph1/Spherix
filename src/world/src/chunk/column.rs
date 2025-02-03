use crate::block::state::BlockState;
use crate::chunk::biome::Biome;
use crate::chunk::handle::{ChunkSectionHandle, RwLockReadGuard, RwLockWriteGuard};
use crate::chunk::heightmap::Heightmaps;
use crate::chunk::palette::container::{create_empty_biome_paletted_container, create_empty_block_paletted_container};
use crate::chunk::palette::{BiomeGlobalPalette, BlockGlobalPalette};
use crate::chunk::pos::ChunkPos;
use crate::chunk::section::ChunkSection;
use crate::chunk::status::ChunkStatus;
use crate::chunk::vector::{Vector3BlockColumn, Vector3BlockSection};
use spherix_math::vector::vec3::Vector3u;
use spherix_math::vector::Vector3;
use spherix_proto::io::BitSet;
use spherix_proto::packet::clientbound::ChunkData;
use spherix_util::nbt::NbtExt;
use std::sync::Arc;

/// Chunk column as described [`here`].
///
/// [`here`]: https://wiki.vg/Chunk_Format#Chunks_columns_and_Chunk_sections
pub struct ChunkColumn {
    /// (x, z) coordinates of the chunk column
    pos: ChunkPos,
    pub status: ChunkStatus,
    sections: Vec<ChunkSectionHandle>,
    pub heightmaps: Heightmaps
}

impl ChunkColumn {
    const Y_MIN: i32 = -64;
    const Y_MAX: i32 = 319;

    pub fn empty(pos: ChunkPos, block_global_palette: Arc<BlockGlobalPalette>, biome_global_palette: Arc<BiomeGlobalPalette>) -> Self {
        let mut sections = Vec::new();
        for i in 0..24 {
            sections.push(
                ChunkSection::new(
                    i,
                    block_global_palette.clone(),
                    biome_global_palette.clone(),
                    create_empty_block_paletted_container(&block_global_palette),
                    create_empty_biome_paletted_container(&biome_global_palette),
                    Some([u8::MAX; 2048]),
                    Some([u8::MAX; 2048])
                ).into()
            );
        }

        Self {
            pos,
            status: ChunkStatus::Empty,
            sections,
            heightmaps: Heightmaps::empty(),
        }
    }

    #[inline]
    pub fn new(pos: ChunkPos, sections: Vec<ChunkSectionHandle>, heightmaps: Heightmaps) -> Self {
        Self {
            pos,
            status: ChunkStatus::Empty,
            sections,
            heightmaps
        }
    }

    pub fn from_nbt(nbt: nbt::Blob, block_palette: Arc<BlockGlobalPalette>, biome_palette: Arc<BiomeGlobalPalette>) -> ChunkColumn {
        let pos_x = *nbt.get("xPos").unwrap().as_byte() as i32;
        let pos_z = *nbt.get("zPos").unwrap().as_byte() as i32;
        let list = nbt.get("sections").unwrap();
        let nbt_sections = list.as_list();

        let mut sections = Vec::with_capacity(24);

        for nbt_section in nbt_sections {
            sections.push(
                ChunkSection::from_nbt(
                    nbt_section,
                    block_palette.clone(),
                    biome_palette.clone()
                ).into()
            );
        }

        Self::new(ChunkPos::new(pos_x, pos_z), sections, Heightmaps::empty())
    }

    pub fn to_load_packet(&self) -> ChunkData {
        let mut sky_light_bitset = BitSet::new();
        sky_light_bitset.clear(0);
        let mut block_light = Vec::new();

        let mut sky_light_empty_bitset = BitSet::new();
        sky_light_empty_bitset.clear(0);

        let mut block_light_bitset = BitSet::new();
        block_light_bitset.clear(0);
        let mut sky_light = Vec::new();

        let mut block_light_empty_bitset = BitSet::new();
        block_light_empty_bitset.clear(0);

        let mut vec = Vec::new();

        for (i, section) in self.sections.iter().enumerate() {
            let guard = section.guarded.read().unwrap();

            vec.extend(guard.to_packet_bytes());

            if guard.sky_light.is_some() {
                sky_light_bitset.set(i + 1);

                if guard.sky_light.unwrap().iter().all(|x| *x == 0) {
                    sky_light_empty_bitset.set(i + 1);
                } else {
                    sky_light_empty_bitset.clear(i + 1);
                }

                sky_light.push(guard.sky_light.unwrap().into());
            } else {
                sky_light_bitset.clear(i + 1);
                sky_light_empty_bitset.clear(i + 1);
            }

            if guard.block_light.is_some() {
                block_light_bitset.set(i + 1);

                if guard.block_light.unwrap().iter().all(|x| *x == 0) {
                    block_light_empty_bitset.set(i + 1);
                } else {
                    block_light_empty_bitset.clear(i + 1);
                }

                block_light.push(guard.block_light.unwrap().into());
            } else {
                block_light_bitset.clear(i + 1);
                block_light_bitset.clear(i + 1);
            }
        }

        sky_light_bitset.clear(self.sections.len() + 1);
        sky_light_empty_bitset.clear(self.sections.len() + 1);

        block_light_bitset.clear(self.sections.len() + 1);
        block_light_empty_bitset.clear(self.sections.len() + 1);

        ChunkData {
            chunk_x: self.pos.x(),
            chunk_z: self.pos.z(),
            heightmaps: self.heightmaps.to_nbt(),
            data: vec.into_boxed_slice(),
            number_of_block_entities: 0.into(),
            trust_edges: true,
            sky_light_mask: sky_light_bitset,
            block_light_mask: block_light_bitset,
            empty_sky_light_mask: sky_light_empty_bitset,
            empty_block_light_mask: block_light_empty_bitset,
            sky_light_arrays: sky_light,
            block_light_arrays: block_light,
        }
    }

    pub fn sections(&self) -> &Vec<ChunkSectionHandle> {
        &self.sections
    }

    pub fn section(&self, index: usize) -> &ChunkSectionHandle {
        &self.sections[index]
    }

    pub fn pos(&self) -> ChunkPos {
        self.pos.clone()
    }

    pub fn block_state(&self, pos: Vector3BlockColumn) -> Option<Arc<BlockState>> {
        #[cfg(debug_assertions)]
        if pos.y() < Self::Y_MIN || pos.y() > Self::Y_MAX {
            panic!("Y {} is out of range. Valid range is [{}; {}].", pos.y(), Self::Y_MIN, Self::Y_MAX);
        }

        let y = (pos.y() + 64) >> 4;
        let section = self.sections.get(y as usize).unwrap();

        let pos = Vector3BlockSection::new(pos.x(), (pos.y() & 0xF) as u32, pos.z() as u32);

        section.guarded.read().unwrap().block_state(pos)
    }

    pub unsafe fn block_state_unguarded(&self, pos: Vector3BlockColumn) -> Option<Arc<BlockState>> {
        #[cfg(debug_assertions)]
        if pos.y() < Self::Y_MIN || pos.y() > Self::Y_MAX {
            panic!("Y {} is out of range. Valid range is [{}; {}].", pos.y(), Self::Y_MIN, Self::Y_MAX);
        }

        let (section_index, section_vector) = pos.to_section_index_and_vector(self.min_build_height());

        let section = self.sections.get(section_index).unwrap();

        section.unguarded.as_mut().unwrap().block_state(section_vector)
    }

    pub fn set_block_state(&mut self, pos: Vector3BlockColumn, state: Arc<BlockState>) {
        #[cfg(debug_assertions)]
        if pos.y() < Self::Y_MIN || pos.y() > Self::Y_MAX {
            panic!("Y out of range");
        }

        let (section_index, section_vector) = pos.to_section_index_and_vector(self.min_build_height());

        let section = self.sections.get_mut(section_index).unwrap();

        section.guarded.write().unwrap().set_block_state(section_vector, state);
    }

    pub unsafe fn set_block_state_unguarded(&mut self, pos: Vector3BlockColumn, state: Arc<BlockState>) {
        #[cfg(debug_assertions)]
        if pos.y() < Self::Y_MIN || pos.y() > Self::Y_MAX {
            panic!("Y out of range");
        }

        let (section_index, section_vector) = pos.to_section_index_and_vector(self.min_build_height());

        let section = self.sections.get_mut(section_index).unwrap();

        section.unguarded.as_mut().unwrap().set_block_state(section_vector, state);
    }

    pub fn biome(&self, pos: Vector3) -> Arc<Biome> {
        let section_index = (pos.y + 16) >> 2;
        let local_y = ((pos.y + 16) % 4) as u32;

        self
            .sections[section_index as usize]
            .guarded
            .read()
            .unwrap()
            .biome(Vector3u::new(pos.x as u32, local_y, pos.z as u32))
            .unwrap()
    }

    pub fn biome2(&self, pos: Vector3) -> Arc<Biome> {
        let i = quart_pos_from_block(self.min_build_height());
        let k = i + quart_pos_from_block(384) - 1;
        let l = pos.y.clamp(i, k);
        let j = Self::section_index(quart_pos_to_block(l));

        self
            .sections[j]
            .guarded
            .read()
            .unwrap()
            .biome(Vector3u::new((pos.x & 3) as u32, (l & 3) as u32, (pos.z & 3) as u32))
            .unwrap()
    }

    pub unsafe fn biome2_unguarded(&self, pos: Vector3) -> Arc<Biome> {
        let i = quart_pos_from_block(self.min_build_height());
        let k = i + quart_pos_from_block(384) - 1;
        let l = pos.y.clamp(i, k);
        let j = Self::section_index(quart_pos_to_block(l));

        self
            .sections[j]
            .unguarded
            .as_ref()
            .unwrap()
            .biome(Vector3u::new((pos.x & 3) as u32, (l & 3) as u32, (pos.z & 3) as u32))
            .unwrap()
    }

    #[inline]
    fn section_index(y: i32) -> usize {
        ((y >> 4) + 4) as usize
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.sections.len()
    }

    #[inline]
    pub fn min_build_height(&self) -> i32 {
        -64
    }

    #[inline]
    pub fn with_safe(&self) -> Ref<RwLockReadGuard> {
        Ref {
            col: self,
            extractor: &|h| {
                RwLockReadGuard(h.guarded.read().unwrap())
            },
        }
    }

    #[inline]
    pub unsafe fn with_unsafe(&self) -> Ref<&ChunkSection> {
        Ref {
            col: self,
            extractor: &|h| {
                unsafe { h.unguarded.as_ref().unwrap() }
            }
        }
    }

    #[inline]
    pub fn with_safe_mut(&mut self) -> RefMut<RwLockWriteGuard> {
        RefMut {
            col: self,
            extractor: &|h| {
                RwLockWriteGuard(h.guarded.write().unwrap())
            },
        }
    }

    #[inline]
    pub fn with_unsafe_mut(&mut self) -> RefMut<&mut ChunkSection> {
        RefMut {
            col: self,
            extractor: &|h| {
                unsafe { h.unguarded.as_mut().unwrap() }
            },
        }
    }
}

#[inline]
fn quart_pos_from_block(block: i32) -> i32 {
    block >> 2
}

#[inline]
fn quart_pos_to_block(block: i32) -> i32 {
    block << 2
}

pub trait ChunkColumnRef {
    fn block_state(&self, at: Vector3BlockColumn) -> Arc<BlockState>;
    fn biome(&self, at: Vector3) -> Arc<Biome>;
}

pub trait ChunkColumnRefMut<'a> {
    fn set_block_state(&'a mut self, at: Vector3BlockColumn, state: Arc<BlockState>);
}

pub struct Ref<'a, R>
where
    R: AsRef<ChunkSection>
{
    col: &'a ChunkColumn,
    extractor: &'a dyn Fn(&'a ChunkSectionHandle) -> R,
}

impl<'a, R> ChunkColumnRef for Ref<'a, R>
where
    R: AsRef<ChunkSection>
{
    fn block_state(&self, at: Vector3BlockColumn) -> Arc<BlockState> {
        let (section_index, section_vector) = at.to_section_index_and_vector(self.col.min_build_height());
        let section = self.col.sections.get(section_index).unwrap();

        let handle = (self.extractor)(section);

        handle.as_ref().block_state(section_vector).unwrap()
    }

    fn biome(&self, at: Vector3) -> Arc<Biome> {
        let i = quart_pos_from_block(self.col.min_build_height());
        let k = i + quart_pos_from_block(384) - 1;
        let l = at.y.clamp(i, k);
        let j = ChunkColumn::section_index(quart_pos_to_block(l));

        self
            .col
            .sections[j]
            .guarded
            .read()
            .unwrap()
            .biome(Vector3u::new((at.x & 3) as u32, (l & 3) as u32, (at.z & 3) as u32))
            .unwrap()
    }
}

pub struct RefMut<'a, R>
where
    R: AsMut<ChunkSection>
{
    col: &'a mut ChunkColumn,
    extractor: &'a dyn Fn(&'a ChunkSectionHandle) -> R,
}

impl<'a, R> ChunkColumnRefMut<'a> for RefMut<'a, R>
where
    R: AsMut<ChunkSection>
{
    fn set_block_state(&'a mut self, at: Vector3BlockColumn, state: Arc<BlockState>) {
        let (section_index, section_vector) = at.to_section_index_and_vector(self.col.min_build_height());
        let section = self.col.sections.get_mut(section_index).unwrap();

        let mut handle = (self.extractor)(section);

        handle.as_mut().set_block_state(section_vector, state);
    }
}

