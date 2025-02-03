use spherix_math::vector::Vector3;
use spherix_world::block::state::BlockState;
use spherix_world::chunk::biome::Biome;
use spherix_world::chunk::column::{ChunkColumnRef, ChunkColumnRefMut};
use spherix_world::chunk::handle::ChunkSectionHandle;
use spherix_world::chunk::pos::ChunkPos;
use spherix_world::chunk::vector::Vector3BlockColumn;
use std::sync::Arc;

pub struct ChunkColumn(spherix_world::chunk::column::ChunkColumn);

impl ChunkColumn {
    #[inline]
    pub fn new(inner: spherix_world::chunk::column::ChunkColumn) -> Self {
        Self(inner)
    }
    
    pub fn inner(&self) -> &spherix_world::chunk::column::ChunkColumn {
        &self.0
    }

    pub fn inner_mut(&mut self) -> &mut spherix_world::chunk::column::ChunkColumn {
        &mut self.0
    }
    
    #[inline]
    pub fn pos(&self) -> ChunkPos {
        self.0.pos()
    }

    #[inline]
    pub fn sections(&self) -> &Vec<ChunkSectionHandle> {
        self.0.sections()
    }

    #[inline]
    pub fn section(&self, index: usize) -> &ChunkSectionHandle {
        self.0.section(index)
    }

    #[inline]
    pub fn with_safe(&self) -> Ref {
        Ref {
            col: self
        }
    }

    #[inline]
    pub unsafe fn with_unsafe(&self) -> Ref {
        Ref {
            col: self
        }
    }

    #[inline]
    pub fn with_safe_mut(&mut self) -> RefMut {
        RefMut {
            col: self
        }
    }

    #[inline]
    pub fn with_unsafe_mut(&mut self) -> RefMut {
        RefMut {
            col: self
        }
    }

    #[inline]
    pub fn min_build_height(&self) -> i32 {
        -64
    }

    #[inline]
    fn section_index(y: i32) -> usize {
        ((y >> 4) + 4) as usize
    }
}

pub struct Ref<'a> {
    col: &'a ChunkColumn
}

impl<'a> ChunkColumnRef for Ref<'a>
{
    fn block_state(&self, at: Vector3BlockColumn) -> Arc<BlockState> {
        self.col.0.block_state(at).unwrap()
    }

    fn biome(&self, at: Vector3) -> Arc<Biome> {
        self.col.0.biome2(at)
    }
}

pub struct RefMut<'a> {
    col: &'a mut ChunkColumn
}

impl<'a> ChunkColumnRefMut<'a> for RefMut<'a> {
    fn set_block_state(&'a mut self, at: Vector3BlockColumn, state: Arc<BlockState>) {
        self.col.0.set_block_state(at, state);
    }
}
