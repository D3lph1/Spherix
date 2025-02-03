use crate::biome::climate::json::BiomeIndex;
use crate::biome::climate::sampler::ClimateSampler;
use spherix_math::vector::Vector3;
use spherix_world::chunk::biome::Biome;
use spherix_world::chunk::palette::BiomeGlobalPalette;
use std::sync::Arc;

pub struct BiomeSampler {
    palette: Arc<BiomeGlobalPalette>,
    index: Arc<BiomeIndex>,
    climate: ClimateSampler
}

impl BiomeSampler {
    pub fn new(palette: Arc<BiomeGlobalPalette>, index: Arc<BiomeIndex>, climate: ClimateSampler) -> Self {
        Self {
            palette,
            index,
            climate,
        }
    }

    pub fn sample(&self, pos: &Vector3) -> Arc<Biome> {
        let point = self.climate.sample(pos);
        let biome = self.index.nearest_neighbor(&point).unwrap();

        self.palette.get_default_obj_by_index(&biome.data).unwrap()
    }
}
