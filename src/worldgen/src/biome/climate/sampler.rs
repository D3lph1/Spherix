use crate::biome::climate::point::ClimatePoint;
use crate::noise::density::cache::quart_pos_to_block;
use crate::noise::density::density::{ContextFiller, DensityFunction, DensityFunctionContext, DensityFunctions};
use spherix_math::vector::Vector3;

pub struct ClimateSampler {
    temperature: DensityFunctions,
    humidity: DensityFunctions,
    continentalness: DensityFunctions,
    erosion: DensityFunctions,
    depth: DensityFunctions,
    weirdness: DensityFunctions,
}

impl ClimateSampler {
    pub fn new(
        temperature: DensityFunctions,
        humidity: DensityFunctions,
        continentalness: DensityFunctions,
        erosion: DensityFunctions,
        depth: DensityFunctions,
        weirdness: DensityFunctions
    ) -> Self {
        Self {
            temperature,
            humidity,
            continentalness,
            erosion,
            depth,
            weirdness
        }
    }

    pub fn sample(&self, pos: &Vector3) -> ClimatePoint {
        let pos = Vector3::new(
            quart_pos_to_block(pos.x),
            quart_pos_to_block(pos.y),
            quart_pos_to_block(pos.z)
        );
        
        let mut ctx = DensityFunctionContext::default();
        ctx.filler = ContextFiller::Slice; // Required for CacheOnce. Mb rewrite the first condition in CacheOnce::sample???

        ClimatePoint {
            temperature: Self::f64_to_i64(self.temperature.sample(pos, &mut ctx)),
            humidity: Self::f64_to_i64(self.humidity.sample(pos, &mut ctx)),
            continentalness: Self::f64_to_i64(self.continentalness.sample(pos, &mut ctx)),
            erosion: Self::f64_to_i64(self.erosion.sample(pos, &mut ctx)),
            depth: Self::f64_to_i64(self.depth.sample(pos, &mut ctx)),
            weirdness: Self::f64_to_i64(self.weirdness.sample(pos, &mut ctx)),
        }
    }
    
    #[inline]
    fn f64_to_i64(val: f64) -> i64 {
        (val * 10000.0) as i64
    }
}
