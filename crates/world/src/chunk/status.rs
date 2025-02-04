pub enum ChunkStatus {
    Empty,
    StructureStarts,
    StructureReferences,
    Biomes,
    Noise,
    Surface,
    Carvers,
    LiquidCarvers,
    Features,
    Light,
    Spawn,
    Heightmaps,
    Full
}

impl ChunkStatus {
    pub fn ordinal(&self) -> usize {
        match self {
            ChunkStatus::Empty => 0,
            ChunkStatus::StructureStarts => 1,
            ChunkStatus::StructureReferences => 2,
            ChunkStatus::Biomes => 3,
            ChunkStatus::Noise => 4,
            ChunkStatus::Surface => 5,
            ChunkStatus::Carvers => 6,
            ChunkStatus::LiquidCarvers => 7,
            ChunkStatus::Features => 8,
            ChunkStatus::Light => 9,
            ChunkStatus::Spawn => 10,
            ChunkStatus::Heightmaps => 11,
            ChunkStatus::Full => 12
        }
    }
}
