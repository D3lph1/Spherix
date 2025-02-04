use spherix_math::vector::vec2::Vector2u;
use spherix_math::vector::{Vector2, Vector3, Vector3u};

/// Represents block position within [`crate::chunk::column::ChunkColumn`].
///
/// This type is primarily a semantic wrapper over [`Vector3`]. It is used to highlight
/// the purpose of a variable / argument.
#[derive(Clone, Copy)]
pub struct Vector3BlockColumn(Vector3);

impl Vector3BlockColumn {
    #[inline]
    pub fn new(x: u32, y: i32, z: u32) -> Self {
        Self(Vector3::new(x as i32, y, z as i32))
    }

    #[inline]
    pub fn x(&self) -> u32 {
        self.0.x() as u32
    }

    #[inline]
    pub fn y(&self) -> i32 {
        self.0.y()
    }

    #[inline]
    pub fn z(&self) -> u32 {
        self.0.z() as u32
    }

    #[inline]
    pub fn to_section_index_and_vector(&self, min_build_height: i32) -> (usize, Vector3BlockSection) {
        let index = (self.y() - min_build_height) >> 4;

        (
            index as usize,
            Vector3BlockSection::new(self.x(), (self.y() & 0xF) as u32, self.z())
        )
    }
}

impl From<Vector3BlockColumn> for Vector3 {
    fn from(value: Vector3BlockColumn) -> Self {
        value.0
    }
}


/// Represents block position within [`crate::chunk::section::ChunkSection`].
///
/// This type is primarily a semantic wrapper over [`Vector3u`]. It is used to highlight
/// the purpose of a variable / argument.
#[derive(Clone, Copy)]
pub struct Vector3BlockSection(Vector3u);

impl Vector3BlockSection {
    #[inline]
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self(Vector3u::new(x, y, z))
    }

    #[inline]
    pub fn x(&self) -> u32 {
        self.0.x()
    }

    #[inline]
    pub fn y(&self) -> u32 {
        self.0.y()
    }

    #[inline]
    pub fn z(&self) -> u32 {
        self.0.z()
    }
}

impl From<Vector3BlockSection> for Vector3u {
    fn from(value: Vector3BlockSection) -> Self {
        value.0
    }
}

#[derive(Clone, Copy)]
pub struct Vector2BlockSection(Vector2u);

impl Vector2BlockSection {
    #[inline]
    pub const fn new(x: u32, z: u32) -> Self {
        Self(Vector2u::new(x, z))
    }
    
    #[inline]
    pub const fn origin() -> Self {
        Self::new(0, 0)
    }
    
    #[inline]
    pub fn x(&self) -> u32 {
        self.0.x()
    }

    #[inline]
    pub fn z(&self) -> u32 {
        self.0.z()
    }
}

impl From<Vector2> for Vector2BlockSection {
    fn from(value: Vector2) -> Self {
        Vector2BlockSection(value & 15)
    }
}

impl From<Vector3BlockColumn> for Vector2BlockSection {
    fn from(value: Vector3BlockColumn) -> Self {
        Vector2BlockSection::new(value.x(), value.z())
    }
}
