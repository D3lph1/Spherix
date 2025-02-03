use crate::aquifer::{DisabledAquifer, FluidPicker};
use crate::biome::climate::json::BiomeIndex;
use crate::biome::climate::sampler::ClimateSampler;
use crate::biome::sampler::BiomeSampler;
use crate::chunk::column::ChunkColumn;
use crate::chunk::noise::NoiseChunk;
use crate::noise::density::cache::{block_to_section_coord, quart_pos_from_block};
use crate::noise::density::density::{ChainMapper, DensityFunctionContext, InterpolatedCollector, SetupInterpolatedMapper, SetupNoiseMapper};
use crate::noise::math::floor_div;
use crate::noise::settings::NoiseSettings;
use crate::rng::XoroShiroPos;
use spherix_math::vector::vec3::Vector3u;
use spherix_math::vector::Vector3;
use spherix_world::block::block::Block;
use spherix_world::chunk::heightmap::{Heightmap, HeightmapType};
use spherix_world::chunk::palette::{BiomeGlobalPalette, BlockGlobalPalette};
use spherix_world::chunk::status::ChunkStatus;
use spherix_world::chunk::vector::{Vector3BlockColumn, Vector3BlockSection};
use std::sync::Arc;

pub trait ChunkGenerator {
    //
}

#[derive(Clone)]
pub struct NoiseBasedChunkGenerator {
    palette: Arc<BlockGlobalPalette>,
    pub biome_palette: Arc<BiomeGlobalPalette>,
    biome_index: Arc<BiomeIndex>,
}

impl NoiseBasedChunkGenerator {
    pub fn new(
        mut noise_settings: NoiseSettings,
        palette: Arc<BlockGlobalPalette>,
        biome_palette: Arc<BiomeGlobalPalette>,
        biome_index: Arc<BiomeIndex>,
        forked: Arc<XoroShiroPos>,
    ) -> (Self, NoiseSettings) {
        let cell_width = noise_settings.cell_width() as usize;
        let cell_count_xz = 16 / cell_width;
        let cell_count_y = floor_div(
            noise_settings.noise_height as i32,
            noise_settings.cell_height() as i32,
        ) as usize;

        let chain_mapper = ChainMapper::new(vec![
            Box::new(SetupNoiseMapper::new(forked)),
            Box::new(SetupInterpolatedMapper::new(cell_count_y, cell_count_xz)),
        ]);

        noise_settings.router = noise_settings.router.map(&chain_mapper);

        let gen = Self {
            palette,
            biome_palette,
            biome_index,
        };

        (gen, noise_settings)
    }

    pub fn do_fill_biomes<'a>(&self, noise_settings: &NoiseSettings, chunk_column: &'a mut ChunkColumn) -> BiomeSampler {
        let chunkpos = chunk_column.pos();
        let p_188006_ = quart_pos_from_block(chunkpos.get_min_block_x());
        let p_188007_ = quart_pos_from_block(chunkpos.get_min_block_z());

        let biome_sampler = BiomeSampler::new(
            self.biome_palette.clone(),
            self.biome_index.clone(),
            ClimateSampler::new(
                noise_settings.router.temperature.clone(),
                noise_settings.router.vegetation.clone(),
                noise_settings.router.continents.clone(),
                noise_settings.router.erosion.clone(),
                noise_settings.router.depth.clone(),
                noise_settings.router.ridges.clone(),
            ),
        );

        // let now = Instant::now();
        // for i in 0..100000 {
        //     let _ = black_box(biome_sampler.sample(&Vector3::new(0, 67, i)));
        // }
        // println!("NOW TIME: {:?}", now.elapsed());

        for section in chunk_column.sections() {
            let section = unsafe { section.unguarded.as_mut().unwrap() };

            let bottom = (section.idx() as i32) - 4;

            let i = quart_pos_from_block(bottom << 4);

            for k in 0..4 {
                for l in 0..4 {
                    for i1 in 0..4 {
                        let x = biome_sampler.sample(
                            &Vector3::new(p_188006_ + k as i32, i + l as i32, p_188007_ + i1 as i32)
                        );

                        section.set_biome(
                            Vector3u::new(k, l, i1),
                            x,
                        );
                    }
                }
            }
        }

        chunk_column.inner_mut().status = ChunkStatus::Biomes;

        biome_sampler
    }

    pub fn do_fill_noise<'a>(&self, noise_settings: &NoiseSettings, chunk_column: &'a mut ChunkColumn, lowest_cell_y: i32, cells_per_chunk_y: i32) -> NoiseChunk {
        let pos = chunk_column.pos();

        let min_chunk_x = pos.get_min_block_x();
        let min_chunk_z = pos.get_min_block_z();

        let cell_width = noise_settings.cell_width() as i32;
        let cell_height = noise_settings.cell_height() as i32;

        let mut ctx = DensityFunctionContext::default();

        ctx.cell_width = noise_settings.cell_width();
        ctx.cell_height = noise_settings.cell_height();
        ctx.cell_count_xz = (16 / cell_width) as u32;
        ctx.cell_count_y = floor_div(
            noise_settings.noise_height as i32,
            noise_settings.cell_height() as i32,
        ) as u32;
        ctx.cell_noise_min_y = floor_div(noise_settings.noise_min_y, cell_height);
        ctx.first_cell_x = floor_div(min_chunk_x, cell_width);
        ctx.first_cell_z = floor_div(min_chunk_z, cell_width);
        ctx.first_noise_x = quart_pos_from_block(min_chunk_x);
        ctx.first_noise_z = quart_pos_from_block(min_chunk_z);
        ctx.noise_size_xz = quart_pos_from_block(ctx.cell_count_xz as i32 * cell_width) as u32;

        let interpolated_collector = InterpolatedCollector::new();

        let router = noise_settings.router.clone().map(&interpolated_collector);

        let interpolators = interpolated_collector.collected.borrow().clone();

        let aquifer = DisabledAquifer::new(FluidPicker::create(noise_settings, &self.palette));

        let mut chunk = NoiseChunk::new(
            ctx,
            router,
            Box::new(aquifer),
            interpolators,
        );

        let cells_per_chunk_xz = 16 / cell_width;

        let mut ocean_floor_heightmap = Heightmap::new(HeightmapType::OceanFloorWg, 384, -64);
        let mut world_surface_heightmap = Heightmap::new(HeightmapType::WorldSurfaceWg, 384, -64);

        for cell_x in 0..cells_per_chunk_xz {
            chunk.advance_cell_x(cell_x);

            for cell_z in 0..cells_per_chunk_xz {
                // section_count = 24
                let mut section_idx = 24 - 1;

                for cell_y in (0..cells_per_chunk_y).rev() {
                    chunk.select_cell_yz(cell_y, cell_z);

                    for in_cell_y in (0..cell_height).rev() {
                        let y = (lowest_cell_y + cell_y) * cell_height + in_cell_y;
                        let in_section_y = (y & 0xF) as u32;
                        let section_index = block_to_section_coord(y) - (-4);

                        let level_bottom_y = block_to_section_coord((section_idx - 4) * 16);
                        if level_bottom_y != section_index {
                            section_idx = section_index;
                        }

                        let y_fraction = in_cell_y as f64 / cell_height as f64;
                        chunk.update_for_y(y, y_fraction);

                        for in_cell_x in 0..cell_width {
                            let x = min_chunk_x + cell_x * cell_width + in_cell_x;
                            let in_section_x = (x & 0xF) as u32;
                            let y_fraction = in_cell_x as f64 / cell_width as f64;

                            chunk.update_for_x(x, y_fraction);

                            for in_cell_z in 0..cell_width {
                                let z = min_chunk_z + cell_z * cell_width + in_cell_z;
                                let in_section_z = (z & 0xF) as u32;
                                let z_fraction = in_cell_z as f64 / cell_width as f64;
                                chunk.update_for_z(z, z_fraction);

                                let mb_block_state = chunk.calculate_interpolated_state();

                                let block_state = if mb_block_state.is_none() {
                                    noise_settings.default_block.clone()
                                } else {
                                    mb_block_state.unwrap()
                                };

                                if block_state.block() != Block::AIR {
                                    let section = unsafe {
                                        chunk_column
                                            .section(section_index as usize)
                                            .unguarded
                                            .as_mut()
                                            .unwrap()
                                    };

                                    section.set_block_state(
                                        Vector3BlockSection::new(in_section_x, in_section_y, in_section_z),
                                        block_state.clone(),
                                    );

                                    let heightmap_pos = Vector3BlockColumn::new(in_section_x, y, in_section_z);

                                    let chunk_column_ref = unsafe { chunk_column.with_unsafe() };
                                    ocean_floor_heightmap.update(&chunk_column_ref, heightmap_pos, block_state.as_ref());
                                    world_surface_heightmap.update(&chunk_column_ref, heightmap_pos, block_state.as_ref());
                                }
                            }
                        }
                    }
                }
            }

            chunk.swap_slices();
        }

        chunk.stop_interpolation();

        chunk_column.inner_mut().heightmaps.ocean_floor_wg = Some(ocean_floor_heightmap);
        chunk_column.inner_mut().heightmaps.world_surface_wg = Some(world_surface_heightmap);

        for i in 0..24 {
            let section = unsafe { chunk_column.section(i).unguarded.as_mut().unwrap() };

            section.sky_light = Some([u8::MAX; 2048]);
        }

        chunk_column.inner_mut().status = ChunkStatus::Noise;

        chunk
    }
}
