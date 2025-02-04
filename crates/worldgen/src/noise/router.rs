use crate::noise::density::density::{DensityFunction, DensityFunctions, Mapper};
use crate::noise::json::Resolver;
use anyhow::anyhow;
use serde_json::{Map, Value};

#[derive(Clone)]
pub struct NoiseRouter {
    pub barrier: DensityFunctions,
    pub fluid_level_floodedness: DensityFunctions,
    pub fluid_level_spread: DensityFunctions,
    pub lava: DensityFunctions,
    //
    pub temperature: DensityFunctions,
    pub vegetation: DensityFunctions,
    pub continents: DensityFunctions,
    pub erosion: DensityFunctions,
    pub depth: DensityFunctions,
    pub ridges: DensityFunctions,
    //
    pub initial_density_without_jaggedness: DensityFunctions,
    pub final_density: DensityFunctions,
    pub vein_toggle: DensityFunctions,
    pub vein_ridged: DensityFunctions,
    pub vein_gap: DensityFunctions
}

impl NoiseRouter {
    pub fn from_json(json: &Value, resolver: &mut Resolver<DensityFunctions>) -> anyhow::Result<Self> {
        let Value::Object(map) = json else {
            return Err(anyhow!("Expected object, found {:?}", json))
        };

        Ok(Self {
            barrier: Self::resolve_sub_df("barrier", map, resolver)?,
            fluid_level_floodedness: Self::resolve_sub_df("fluid_level_floodedness", map, resolver)?,
            fluid_level_spread: Self::resolve_sub_df("fluid_level_spread", map, resolver)?,
            lava: Self::resolve_sub_df("lava", map, resolver)?,
            temperature: Self::resolve_sub_df("temperature", map, resolver)?,
            vegetation: Self::resolve_sub_df("vegetation", map, resolver)?,
            continents: Self::resolve_sub_df("continents", map, resolver)?,
            erosion: Self::resolve_sub_df("erosion", map, resolver)?,
            depth: Self::resolve_sub_df("depth", map, resolver)?,
            ridges: Self::resolve_sub_df("ridges", map, resolver)?,
            initial_density_without_jaggedness: Self::resolve_sub_df("initial_density_without_jaggedness", map, resolver)?,
            final_density: Self::resolve_sub_df("final_density", map, resolver)?,
            vein_toggle: Self::resolve_sub_df("vein_toggle", map, resolver)?,
            vein_ridged: Self::resolve_sub_df("vein_ridged", map, resolver)?,
            vein_gap: Self::resolve_sub_df("vein_gap", map, resolver)?,
        })
    }

    fn resolve_sub_df(key: &str, map: &Map<String, Value>, resolver: &mut Resolver<DensityFunctions>) -> anyhow::Result<DensityFunctions> {
        if map.contains_key(key) {
            resolver.resolve(map.get(key).unwrap())
        } else {
            Err(anyhow!("No \"{}\" key", key))
        }
    }

    pub fn map<M: Mapper>(self, mapper: &M) -> NoiseRouter {
        NoiseRouter {
            barrier: self.barrier.map(mapper),
            fluid_level_floodedness: self.fluid_level_floodedness.map(mapper),
            fluid_level_spread: self.fluid_level_spread.map(mapper),
            lava: self.lava.map(mapper),
            temperature: self.temperature.map(mapper),
            vegetation: self.vegetation.map(mapper),
            continents: self.continents.map(mapper),
            erosion: self.erosion.map(mapper),
            depth: self.depth.map(mapper),
            ridges: self.ridges.map(mapper),
            initial_density_without_jaggedness: self.initial_density_without_jaggedness.map(mapper),
            final_density: self.final_density.map(mapper),
            vein_toggle: self.vein_toggle.map(mapper),
            vein_ridged: self.vein_ridged.map(mapper),
            vein_gap: self.vein_gap.map(mapper),
        }
    }
}
