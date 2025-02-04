use crate::block::packed::PackedArray;
use crate::block::state::BlockState;
use crate::chunk::column::ChunkColumnRef;
use crate::chunk::vector::block::Vector2BlockSection;
use crate::chunk::vector::Vector3BlockColumn;
use spherix_math::vector::Vector2;
use spherix_util::math::smallest_encompassing_log2;
use std::collections::HashMap;

pub struct Heightmap
{
    ty: HeightmapType,
    min_build_height: i32,
    pub data: PackedArray,
}

impl Heightmap
{
    pub fn new(ty: HeightmapType, world_height: i32, min_build_height: i32) -> Self {
        Self {
            ty,
            min_build_height,
            data: PackedArray::zeros(smallest_encompassing_log2((world_height + 1) as u32) as usize, 256),
        }
    }

    pub fn update<C>(&mut self, chunk: &C, at: Vector3BlockColumn, block: &BlockState) -> bool
    where
        C: ChunkColumnRef
    {
        let first_available_y = self.first_available_y(at.into());

        if at.y() < first_available_y - 2 {
            false
        } else {
            if self.ty.is_opaque(block) {
                if at.y() >= first_available_y {
                    self.set_height(at.into(), (at.y() + 1 - self.min_build_height) as u16);
                    return true;
                }
            } else if first_available_y - 1 == at.y() {
                for j in (self.min_build_height..=at.y() - 1).rev() {
                    let curr_block = chunk.block_state(Vector3BlockColumn::new(at.x(), j, at.z()));
                    if self.ty.is_opaque(curr_block.as_ref()) {
                        self.set_height(at.into(), (j + 1 - self.min_build_height) as u16);

                        return true;
                    }
                }

                self.set_height(at.into(), 0);
                return true;
            }

            false
        }
    }

    #[inline]
    pub fn height(&self, at: Vector2) -> i32 {
        self.first_available_y(at.into()) - 1
    }

    #[inline]
    pub fn height_section(&self, at: Vector2BlockSection) -> i32 {
        self.first_available_y(at) - 1
    }

    #[inline]
    fn first_available_y(&self, at: Vector2BlockSection) -> i32 {
        self.data.get(Self::index(at)) as i32 + self.min_build_height
    }

    #[inline]
    fn set_height(&mut self, at: Vector2BlockSection, height: u16) {
        self.data.set(Self::index(at), height);
    }

    #[inline]
    fn index(at: Vector2BlockSection) -> usize {
        (at.x() + at.z() * 16) as usize
    }
}

pub enum HeightmapType {
    WorldSurfaceWg,
    WorldSurface,
    OceanFloorWg,
    OceanFloor,
    MotionBlocking,
    MotionBlockingNoLeaves
}

impl HeightmapType {
    fn is_opaque(&self, block: &BlockState) -> bool {
        match self {
            HeightmapType::WorldSurfaceWg => Self::is_not_air(block),
            HeightmapType::WorldSurface => Self::is_not_air(block),
            HeightmapType::OceanFloorWg => Self::is_material_motion_blocking(block),
            HeightmapType::OceanFloor => Self::is_material_motion_blocking(block),
            HeightmapType::MotionBlocking => {
                todo!()
            },
            HeightmapType::MotionBlockingNoLeaves => {
                todo!()
            }
        }
    }

    #[inline]
    fn is_not_air(block: &BlockState) -> bool {
        !block.block().properties().is_air
    }

    #[inline]
    fn is_material_motion_blocking(block: &BlockState) -> bool {
        block.block().properties().material().blocks_motion
    }
}

pub struct Heightmaps {
    pub world_surface_wg: Option<Heightmap>,
    pub world_surface: Option<Heightmap>,
    pub ocean_floor_wg: Option<Heightmap>,
    pub ocean_floor: Option<Heightmap>,
    pub motion_blocking: Option<Heightmap>,
    pub motion_blocking_no_leaves: Option<Heightmap>,
}

impl Heightmaps {
    pub fn empty() -> Self {
        Self {
            world_surface_wg: None,
            world_surface: None,
            ocean_floor_wg: None,
            ocean_floor: None,
            motion_blocking: None,
            motion_blocking_no_leaves: None,
        }
    }

    pub fn to_nbt(&self) -> nbt::Blob {
        let mut blob = nbt::Blob::new();

        if self.world_surface.is_some() {
            blob.insert("WORLD_SURFACE", nbt::Value::Compound(HashMap::new())).unwrap();
        }

        if self.ocean_floor.is_some() {
            blob.insert("OCEAN_FLOOR", nbt::Value::Compound(HashMap::new())).unwrap();
        }

        if self.motion_blocking.is_some() {
            blob.insert("MOTION_BLOCKING", nbt::Value::Compound(HashMap::new())).unwrap();
        }

        if self.motion_blocking_no_leaves.is_some() {
            blob.insert("MOTION_BLOCKING_NO_LEAVES", nbt::Value::Compound(HashMap::new())).unwrap();
        }

        blob
    }
}
