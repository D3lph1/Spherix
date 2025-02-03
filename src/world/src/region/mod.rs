use crate::region::pos::ChunkWithinRegionPos;

pub mod pos;
pub mod anvil;

pub trait RegionFile {
    fn read(&mut self, chunk_pos: ChunkWithinRegionPos) -> Option<nbt::Blob>;

    fn does_chunk_exist(&mut self, chunk_pos: ChunkWithinRegionPos) -> bool;
}
