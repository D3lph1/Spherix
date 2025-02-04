use serde_json::Value;
use spherix_world::chunk::palette::{create_biome_global_palette_from_json, create_block_global_palette_from_json, BiomeGlobalPalette, BlockGlobalPalette};
use spherix_worldgen::biome::climate::json::{create_biome_index_from_json, BiomeIndex};
use spherix_worldgen::noise::json::value_resolver::{CachedValueResolver, CascadeValueResolver, FilesystemValueResolver};
use spherix_worldgen::noise::json::{deserializers, Resolver};
use spherix_worldgen::noise::settings::NoiseSettings;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

pub fn create_block_global_palette() -> Arc<BlockGlobalPalette> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../generated/reports/blocks.json");

    let f = std::fs::read_to_string(path).unwrap();
    let json: Value = serde_json::from_str(&f).unwrap();

    Arc::new(create_block_global_palette_from_json(json))
}

pub fn create_biome_global_palette() -> Arc<BiomeGlobalPalette> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../generated/registry_codec.json");

    let f = std::fs::read_to_string(path).unwrap();
    let json: Value = serde_json::from_str(&f).unwrap();

    let biomes = json
        .as_object()
        .unwrap()
        .get("minecraft:worldgen/biome")
        .unwrap()
        .as_object()
        .unwrap()
        .get("value")
        .unwrap();

    Arc::new(create_biome_global_palette_from_json(biomes))
}

pub fn create_noise_settings(palette: Arc<BlockGlobalPalette>) -> NoiseSettings {
    let mut df_resolver = Resolver::new(
        deserializers(),
        Box::new(
            CachedValueResolver::new(
                CascadeValueResolver::new(
                    vec![
                        Box::new(FilesystemValueResolver::new(
                            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../generated/data/minecraft/worldgen/density_function")
                        )),
                        Box::new(FilesystemValueResolver::new(
                            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../generated/data/minecraft/worldgen/noise")
                        ))
                    ]
                )
            )
        )
    );

    let file = BufReader::new(
        File::open(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../generated/data/minecraft/worldgen/noise_settings/overworld.json")).unwrap()
    );
    let json = serde_json::from_reader(file).unwrap();

    NoiseSettings::from_json(&json, &mut df_resolver, palette.clone()).unwrap()
}

pub fn create_biome_index() -> Arc<BiomeIndex> {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../../generated/reports/biome_parameters/minecraft/overworld.json");

    let f = std::fs::read_to_string(path).unwrap();

    Arc::new(create_biome_index_from_json(f).unwrap())
}
