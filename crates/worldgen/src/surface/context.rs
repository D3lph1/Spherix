use crate::biome::gradient::LazyCachedBiomeGradient;
use crate::noise::density::density::DebugTree;
use crate::noise::density::noise::NoiseHolder;
use crate::noise::perlin::octave::{MultiOctaveNoiseFactory, MultiOctaveNoiseParameters};
use crate::noise::perlin::DefaultNoise;
use crate::rng::{RngForkable, RngPos, XoroShiroPos};
use crate::surface::level::SurfaceLevel;
use spherix_world::chunk::heightmap::Heightmaps;
use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Context<'a> {
    pub gen_ctx: WorldGenerationContext,
    pub heightmaps: &'a Heightmaps,
    pub surface_level: SurfaceLevel<'a>,
    pub biome_gradient: Option<&'a LazyCachedBiomeGradient<'a>>,
    pub entropy_bag: Arc<EntropyBag>,
    pub debug_tree: Option<DebugTree<String>>,
}

impl<'a> Context<'a> {
    pub fn new(
        gen_ctx: WorldGenerationContext,
        heightmaps: &'a Heightmaps,
        surface_level: SurfaceLevel<'a>,
        entropy_bag: Arc<EntropyBag>,
    ) -> Self {
        Self {
            gen_ctx,
            heightmaps,
            surface_level,
            biome_gradient: None,
            entropy_bag,
            debug_tree: None,
        }
    }

    pub fn update_xz(&mut self, x: i32, z: i32) {
        self.surface_level.update_xz(x, z)
    }
    
    pub fn update_y(&mut self, stone_depth_above: i32, stone_depth_below: i32, water_height: i32, y: i32, biome_gradient: &'a LazyCachedBiomeGradient) {
        self.surface_level.update_y(
            y,
            water_height,
            stone_depth_below,
            stone_depth_above,
        );
        self.biome_gradient = Some(biome_gradient);
    }
}

pub fn memorize<T, F>(f: F) -> impl Fn() -> T
where
    T: Clone,
    F: Clone + FnOnce() -> T
{
    let once = OnceCell::new();
    move || {
        let cell = once.get_or_init(f.clone()).clone();
        cell
    }
}

pub struct WorldGenerationContext {
    pub height: i32,
    pub min_y: i32
}

pub struct EntropyBag {
    pub rng: Arc<XoroShiroPos>,
    pub noises: Noises,
    noise_cache: Mutex<HashMap<String, Arc<DefaultNoise>>>,
    pos_cache: Mutex<HashMap<String, Arc<XoroShiroPos>>>,
}

impl EntropyBag {
    pub fn new(rng: Arc<XoroShiroPos>, noises: Noises) -> Self {
        Self {
            rng,
            noises,
            noise_cache: Mutex::new(HashMap::new()),
            pos_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn get_noise(&self, name: String) -> Option<Arc<DefaultNoise>> {
        self.noise_cache.lock().unwrap().get(&name).cloned()
    }

    pub fn get_or_create_noise(&self, name: String, params: &MultiOctaveNoiseParameters) -> Arc<DefaultNoise> {
        self.noise_cache
            .lock()
            .unwrap()
            .entry(name.clone())
            .or_insert_with(|| {
                Arc::new(
                    DefaultNoise::create(
                        &mut self.rng.by_hash(name),
                        &params.amplitudes,params.first_octave
                    )
                )
            })
            .clone()
    }

    pub fn get_or_create_rng_pos(&self, tag: String) -> Arc<XoroShiroPos> {
        self.pos_cache
            .lock()
            .unwrap()
            .entry(tag.clone())
            .or_insert_with(|| Arc::new(self.rng.by_hash(tag).fork_pos()))
            .clone()
    }
}

pub struct Noises {
    pub clay_bands_offset: Arc<NoiseHolder<DefaultNoise>>,
    pub badlands_pillar: Arc<NoiseHolder<DefaultNoise>>,
    pub badlands_pillar_roof: Arc<NoiseHolder<DefaultNoise>>,
    pub badlands_surface: Arc<NoiseHolder<DefaultNoise>>,
    pub iceberg_pillar: Arc<NoiseHolder<DefaultNoise>>,
    pub iceberg_pillar_roof: Arc<NoiseHolder<DefaultNoise>>,
    pub iceberg_surface: Arc<NoiseHolder<DefaultNoise>>,
    pub surface: Arc<NoiseHolder<DefaultNoise>>,
    pub surface_secondary: Arc<NoiseHolder<DefaultNoise>>,
}
