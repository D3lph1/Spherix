use crate::biome::sampler::BiomeSampler;
use crate::chunk::column::ChunkColumn;
use crate::noise::density::cache::quart_pos_to_section;
use gxhash::GxBuildHasher;
use spherix_math::vector::Vector3;
use spherix_world::chunk::biome::Biome;
use spherix_world::chunk::column::ChunkColumnRef;
use spherix_world::chunk::pos::ChunkPos;
use spherix_world::chunk::status::ChunkStatus;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct BiomeAccessor {
    pub current_chunk: Arc<ChunkColumn>,
    pub generator_cache: Arc<RwLock<HashMap<ChunkPos, Arc<ChunkColumn>, GxBuildHasher>>>,
    pub chunks: Arc<RwLock<HashMap<ChunkPos, Option<Arc<ChunkColumn>>, GxBuildHasher>>>,
    pub sampler: BiomeSampler,
}

impl BiomeAccessor {
    pub fn biome_at(&self, at: &Vector3) -> Arc<Biome> {
        let chunk_pos = ChunkPos::new(quart_pos_to_section(at.x), quart_pos_to_section(at.z));

        let guard = self.generator_cache.read().unwrap();
        if guard.contains_key(&chunk_pos) {
            let chunk = guard.get(&chunk_pos).unwrap().clone();
            drop(guard);

            if chunk.inner().status.ordinal() >= ChunkStatus::Biomes.ordinal() {
                return unsafe { chunk.with_unsafe().biome(at.clone()) }
            }
        }

        let guard = self.chunks.read().unwrap();
        if guard.contains_key(&chunk_pos) {
            let chunk = guard
                .get(&chunk_pos)
                .unwrap();

            if chunk.is_some() {
                let chunk = chunk.as_ref().unwrap().clone();
                drop(guard);

                return unsafe { chunk.with_unsafe().biome(at.clone()) }
            }
        }

        drop(guard);

        self.sampler.sample(at)
    }
}
