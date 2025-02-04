use crate::chunk::section::ChunkSection;
use std::sync::RwLock;

/// Pair of safe (guarded) and unsafe (unguarded) pointers to the [`ChunkSection`].
/// The unsafe one is always valid until this [`ChunkSectionHandle`] is not dropped.
pub struct ChunkSectionHandle {
    pub guarded: RwLock<Box<ChunkSection>>,
    pub unguarded: *mut ChunkSection,
}

unsafe impl Send for ChunkSectionHandle {}

unsafe impl Sync for ChunkSectionHandle {}

impl ChunkSectionHandle {
    #[inline]
    pub fn new(chunk: ChunkSection) -> Self {
        let mut b = Box::new(chunk);

        Self {
            unguarded: &mut *b as *mut ChunkSection,
            guarded: RwLock::new(b),
        }
    }
}

impl From<ChunkSection> for ChunkSectionHandle {
    fn from(value: ChunkSection) -> Self {
        ChunkSectionHandle::new(value)
    }
}

pub struct RwLockReadGuard<'a> (pub std::sync::RwLockReadGuard<'a, Box<ChunkSection>>);

impl<'a> AsRef<ChunkSection> for RwLockReadGuard<'a> {
    fn as_ref(&self) -> &ChunkSection {
        &**self.0
    }
}

pub struct RwLockWriteGuard<'a> (pub std::sync::RwLockWriteGuard<'a, Box<ChunkSection>>);

impl<'a> AsMut<ChunkSection> for RwLockWriteGuard<'a> {
    fn as_mut(&mut self) -> &mut ChunkSection {
        &mut **self.0
    }
}
