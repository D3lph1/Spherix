use std::ops::{Add, Sub};

use crate::chunk::pos::ChunkPos;
use spherix_math::vector::Vector2;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RegionPos(Vector2);

impl RegionPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self(Vector2::new(x, z))
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.0.y
    }
}

impl From<Vector2> for RegionPos {
    fn from(value: Vector2) -> Self {
        Self(value)
    }
}

impl From<ChunkPos> for RegionPos {
    fn from(value: ChunkPos) -> Self {
        Self::new(
            f32::floor(value.x() as f32 / 32.0) as i32,
            f32::floor(value.z() as f32 / 32.0) as i32,
        )
    }
}

impl Add for RegionPos {
    type Output = RegionPos;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        RegionPos(self.0.add(rhs.0))
    }
}

impl Sub for RegionPos {
    type Output = RegionPos;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        RegionPos(self.0.sub(rhs.0))
    }
}

#[derive(Clone)]
pub struct ChunkWithinRegionPos(Vector2);

impl ChunkWithinRegionPos {
    pub fn new(x: i32, z: i32) -> Self {
        Self(Vector2::new(x, z))
    }

    #[inline]
    pub fn x(&self) -> i32 {
        self.0.x
    }

    #[inline]
    pub fn z(&self) -> i32 {
        self.0.y
    }
}

impl From<Vector2> for ChunkWithinRegionPos {
    fn from(value: Vector2) -> Self {
        Self(value)
    }
}

impl From<ChunkPos> for ChunkWithinRegionPos {
    fn from(value: ChunkPos) -> Self {
        Self::new(value.x() & 0x1F, value.z() & 0x1F)
    }
}
