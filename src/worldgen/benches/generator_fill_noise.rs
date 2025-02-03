mod shared;
mod pprof;

use crate::pprof::FlamegraphProfiler;
use crate::shared::{create_biome_global_palette, create_biome_index, create_block_global_palette, create_noise_settings};
use criterion::{criterion_group, criterion_main, Criterion};
use spherix_world::chunk::column::ChunkColumn;
use spherix_world::chunk::pos::ChunkPos;
use spherix_worldgen::chunk::column::ChunkColumn as WorldgenColumnColumn;
use spherix_worldgen::chunk::generator::NoiseBasedChunkGenerator;
use spherix_worldgen::rng::{RngForkable, XoroShiro};
use std::sync::Arc;

pub fn benchmark(c: &mut Criterion) {
    let block_palette = create_block_global_palette();
    let biome_palette = create_biome_global_palette();

    let noise_settings = create_noise_settings(block_palette.clone());
    let biome_index = create_biome_index();

    let chunk = ChunkColumn::empty(
        ChunkPos::new(0, 0),
        block_palette.clone(),
        biome_palette.clone(),
    );

    let mut worldgen_chunk = WorldgenColumnColumn::new(chunk);

    let mut rng = XoroShiro::new(200);
    let forked = Arc::new(rng.fork_pos());

    let (gen, noise_settings) = NoiseBasedChunkGenerator::new(
        noise_settings,
        block_palette,
        biome_palette,
        biome_index,
        forked.clone()
    );

    c.bench_function("generator_fill_noise", |b| {
        b.iter(|| {
            gen.do_fill_noise(&noise_settings, &mut worldgen_chunk, -8, 48);
        })
    });
}

criterion_group!(
    name = benches;
    config = Criterion::default().with_profiler(FlamegraphProfiler::new(100));
    targets = benchmark
);
criterion_main!(benches);
