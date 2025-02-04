use crate::dimension::DimensionKind;
use spherix_math::vector::{Vector2, Vector3f, VectorPlain};
use std::ops::{Add, Sub};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkPos(Vector2);

impl ChunkPos {
    pub const INVALID: ChunkPos = ChunkPos::new(1875066, 1875066);

    pub const fn new(x: i32, z: i32) -> Self {
        Self(Vector2::new(x, z))
    }

    #[inline]
    pub fn extract_x(val: i64) -> i32 {
        (val & 4294967295) as i32
    }

    #[inline]
    pub fn extract_z(val: i64) -> i32 {
        (val >> 32 & 4294967295) as i32
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.0.y
    }
    
    #[inline]
    pub fn get_min_block_x(&self) -> i32 {
        self.x() << 4 // section_to_block_coord(self.x())
    }

    #[inline]
    pub fn get_min_block_z(&self) -> i32 {
        self.z() << 4 // section_to_block_coord(self.z())
    }
}

impl From<Vector2> for ChunkPos {
    fn from(value: Vector2) -> Self {
        Self(value)
    }
}

impl From<ChunkPos> for Vector2 {
    fn from(value: ChunkPos) -> Self {
        value.0
    }
}

impl From<Vector3f> for ChunkPos {
    fn from(value: Vector3f) -> Self {
        Self::new(f32::floor(value.x as f32 / 16.0) as i32, f32::floor(value.z as f32 / 16.0) as i32)
    }
}

impl From<ChunkPos> for i64 {
    fn from(value: ChunkPos) -> i64 {
        value.x() as i64 & 4294967295 | (value.z() as i64 & 4294967295) << 32
    }
}

impl VectorPlain for ChunkPos {
    fn new_from(&self, x: i32, z: i32) -> Self {
        Self(Vector2::new(x, z))
    }

    #[inline]
    fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    fn z(&self) -> i32 {
        self.0.y
    }
}

impl Add for ChunkPos {
    type Output = ChunkPos;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        ChunkPos(self.0.add(rhs.0))
    }
}

impl Sub for ChunkPos {
    type Output = ChunkPos;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        ChunkPos(self.0.sub(rhs.0))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GlobalChunkPos {
    pub dim: DimensionKind,
    pub vec: ChunkPos,
}

impl GlobalChunkPos {
    #[inline]
    pub fn new(dim: DimensionKind, vec: ChunkPos) -> Self {
        Self {
            dim,
            vec,
        }
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.vec.x()
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.vec.z()
    }
    
    #[inline]
    pub fn dim(&self) -> DimensionKind {
        self.dim.clone()
    }
}

impl VectorPlain for GlobalChunkPos {
    #[inline]
    fn new_from(&self, x: i32, y: i32) -> Self {
        Self::new(self.dim.clone(), ChunkPos::new(x, y))
    }

    #[inline]
    fn x(&self) -> i32 {
        self.vec.x()
    }

    #[inline]
    fn z(&self) -> i32 {
        self.vec.z()
    }
}

impl Add for GlobalChunkPos {
    type Output = GlobalChunkPos;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        GlobalChunkPos::new(self.dim, self.vec.add(rhs.vec))
    }
}

impl Sub for GlobalChunkPos {
    type Output = GlobalChunkPos;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        GlobalChunkPos::new(self.dim, self.vec.sub(rhs.vec))
    }
}
