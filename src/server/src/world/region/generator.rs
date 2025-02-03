use crate::perf::worker::{ForceSend, StaticTaskHandle};
use crate::world::region::worker::ChunkTask;
use flume::Sender;
use gxhash::GxBuildHasher;
use spherix_world::chunk::column::ChunkColumn;
use spherix_world::chunk::palette::{BiomeGlobalPalette, BlockGlobalPalette};
use spherix_world::chunk::pos::ChunkPos;
use spherix_worldgen::biome::accessor::BiomeAccessor;
use spherix_worldgen::biome::climate::json::create_biome_index_from_json;
use spherix_worldgen::chunk::column::ChunkColumn as WorldgenChunkColumn;
use spherix_worldgen::chunk::generator::NoiseBasedChunkGenerator;
use spherix_worldgen::noise::density::noise::NoiseHolder;
use spherix_worldgen::noise::json::resolvable::Resolvable;
use spherix_worldgen::noise::json::value_resolver::{CachedValueResolver, CascadeValueResolver, FilesystemValueResolver, NoReturnValueResolver};
use spherix_worldgen::noise::json::{deserializers, Resolver};
use spherix_worldgen::noise::perlin::DefaultNoise;
use spherix_worldgen::noise::settings::NoiseSettings;
use spherix_worldgen::rng::{RngForkable, RngPos, XoroShiro};
use spherix_worldgen::surface::bands::generate_bands;
use spherix_worldgen::surface::condition_factory::ConditionFactories;
use spherix_worldgen::surface::context::{EntropyBag, Noises};
use spherix_worldgen::surface::json::{condition_deserializers, rule_deserializers};
use spherix_worldgen::surface::materializer::SurfaceMaterializer;
use spherix_worldgen::surface::rule_factory::RuleFactories;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub struct RegionGeneratorWorkerHandler {
    block_global_palette: Arc<BlockGlobalPalette>,
    biome_global_palette: Arc<BiomeGlobalPalette>,
    generator: NoiseBasedChunkGenerator,
    surface_materializer: SurfaceMaterializer,
    cache: Arc<RwLock<HashMap<ChunkPos, Arc<WorldgenChunkColumn>, GxBuildHasher>>>,
    chunk_tx: Sender<Arc<WorldgenChunkColumn>>,
}

impl RegionGeneratorWorkerHandler {
    pub fn new(
        palette: Arc<BlockGlobalPalette>,
        biome_global_palette: Arc<BiomeGlobalPalette>,
        chunk_cache: Arc<RwLock<HashMap<ChunkPos, Arc<WorldgenChunkColumn>, GxBuildHasher>>>,
        chunk_tx: Sender<Arc<WorldgenChunkColumn>>
    ) -> (Self, NoiseSettings, Arc<EntropyBag>, Arc<RuleFactories>) {
        let mut df_resolver = Resolver::new(
            deserializers(),
            Box::new(
                CachedValueResolver::new(
                    CascadeValueResolver::new(
                        vec![
                            Box::new(FilesystemValueResolver::new(
                                PathBuf::from("./generated/data/minecraft/worldgen/density_function")
                            )),
                            Box::new(FilesystemValueResolver::new(
                                PathBuf::from("./generated/data/minecraft/worldgen/noise")
                            ))
                        ]
                    )
                )
            )
        );

        let file = BufReader::new(
            File::open(PathBuf::from("./generated/data/minecraft/worldgen/noise_settings/overworld.json")).unwrap()
        );
        let json = serde_json::from_reader(file).unwrap();

        let noise_settings = NoiseSettings::from_json(&json, &mut df_resolver, palette.clone()).unwrap();

        let path = "generated/reports/biome_parameters/minecraft/overworld.json";
        let f = std::fs::read_to_string(path).unwrap();

        let biome_index = Arc::new(create_biome_index_from_json(f).unwrap());
        
        let mut rng = XoroShiro::new(1);
        let forked = Arc::new(rng.fork_pos());

        let (gen, noise_settings) = NoiseBasedChunkGenerator::new(
            noise_settings,
            palette.clone(),
            biome_global_palette.clone(),
            biome_index,
            forked.clone()
        );

        let condition_resolver = Resolver::new(
            condition_deserializers(),
            Box::new(
                FilesystemValueResolver::new(
                    PathBuf::from("generated/data/minecraft/worldgen/noise")
                )
            )
        );

        let entropy_bag = EntropyBag::new(
            forked.clone(),
            Noises {
                clay_bands_offset: deserialize_noise(&forked, &condition_resolver, "minecraft:clay_bands_offset"),
                badlands_pillar: deserialize_noise(&forked, &condition_resolver, "minecraft:badlands_pillar"),
                badlands_pillar_roof: deserialize_noise(&forked, &condition_resolver, "minecraft:badlands_pillar_roof"),
                badlands_surface: deserialize_noise(&forked, &condition_resolver, "minecraft:badlands_surface"),
                iceberg_pillar: deserialize_noise(&forked, &condition_resolver, "minecraft:iceberg_pillar"),
                iceberg_pillar_roof: deserialize_noise(&forked, &condition_resolver, "minecraft:iceberg_pillar_roof"),
                iceberg_surface: deserialize_noise(&forked, &condition_resolver, "minecraft:iceberg_surface"),
                surface: deserialize_noise(&forked, &condition_resolver, "minecraft:surface"),
                surface_secondary: deserialize_noise(&forked, &condition_resolver, "minecraft:surface_secondary"),
            }
        );

        let surface_resolver = Resolver::new(
            rule_deserializers(
                condition_resolver,
                palette.clone(),
                generate_bands(&mut forked.by_hash("minecraft:clay_bands".to_owned()), palette.clone())
            ),
            Box::new(NoReturnValueResolver)
        );

        let surface_rule = json.get("surface_rule").unwrap();
        let surface_rule_factory = surface_resolver.resolve(surface_rule).unwrap();

        let surface_materializer = SurfaceMaterializer::new(palette.clone());

        (
            Self {
                block_global_palette: palette,
                biome_global_palette,
                generator: gen,
                surface_materializer,
                cache: chunk_cache,
                chunk_tx,
            },
            noise_settings,
            Arc::new(entropy_bag),
            Arc::new(surface_rule_factory)
        )
    }

    pub fn generate(
        &self,
        noise_settings: NoiseSettings,
        entropy_bag: Arc<EntropyBag>,
        rule_factory: Arc<RuleFactories>,
        pos: ChunkPos
    ) {
        let chunk = ChunkColumn::empty(
            pos.clone(),
            self.block_global_palette.clone(),
            self.biome_global_palette.clone(),
        );

        let mut worldgen_chunk = WorldgenChunkColumn::new(chunk);

        let biome_sampler = self.generator.do_fill_biomes(
            &noise_settings,
            &mut worldgen_chunk
        );

        let arc = Arc::new(worldgen_chunk);
        self.cache.write().unwrap().insert(pos, arc.clone());
        let ptr_mut = arc.as_ref() as *const WorldgenChunkColumn as *mut WorldgenChunkColumn;
        let ref_mut = unsafe { ptr_mut.as_mut().unwrap() };
        
        let now = Instant::now();
        let noise_chunk = self.generator.do_fill_noise(
            &noise_settings,
            ref_mut,
            -8,
            48
        );
        println!("TIMING for ({}, {}): {:?}", arc.pos().x(), arc.pos().z(), now.elapsed());

        self.surface_materializer.materialize(
            &noise_settings,
            entropy_bag,
            rule_factory,
            noise_chunk,
            BiomeAccessor {
                current_chunk: arc.clone(),
                generator_cache: self.cache.clone(),
                chunks: Arc::new(Default::default()),
                sampler: biome_sampler,
            },
            ref_mut,
        );

        self.chunk_tx.send(arc).unwrap();
    }
}

impl StaticTaskHandle<ChunkTask, (ForceSend<NoiseSettings>, Arc<EntropyBag>, Arc<RuleFactories>)> for RegionGeneratorWorkerHandler {
    fn handle(&self, task: ChunkTask, thread_local_state: (ForceSend<NoiseSettings>, Arc<EntropyBag>, Arc<RuleFactories>)) {
        match task {
            ChunkTask::Load(task) => self.generate(
                unsafe { thread_local_state.0.into_inner() },
                thread_local_state.1,
                thread_local_state.2,
                task.0
            )
        }
    }
}

fn deserialize_noise<R: RngPos, F: AsRef<R>>(rng: F, resolver: &Resolver<ConditionFactories>, name: &str) -> Arc<NoiseHolder<DefaultNoise>> {
    resolver.contextual_name.set(Some(name.to_owned()));

    let noise = NoiseHolder::resolve(
        &resolver.resolve_value(name.to_owned()).unwrap(),
        &resolver
    )
        .unwrap();
    
    noise
        .with_rng(&mut rng.as_ref().by_hash(name.to_owned()))
        .into()
}
