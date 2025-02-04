use crate::biome::accessor::BiomeAccessor;
use crate::biome::gradient::{BiomeGradient, LazyCachedBiomeGradient};
use crate::biome::temperature::should_melt_frozen_ocean_iceberg_slightly;
use crate::chunk::column::ChunkColumn;
use crate::chunk::noise::NoiseChunk;
use crate::noise::math::floor;
use crate::noise::perlin::Noise;
use crate::noise::settings::NoiseSettings;
use crate::rng::{Rng, RngPos};
use crate::surface::context::{Context, EntropyBag, WorldGenerationContext};
use crate::surface::level::SurfaceLevel;
use crate::surface::rule::Rule;
use crate::surface::rule_factory::{RuleFactories, RuleFactory};
use num_traits::abs;
use spherix_math::vector::{Vector3, Vector3f};
use spherix_world::block::block::Block;
use spherix_world::block::material::Material;
use spherix_world::block::state::BlockState;
use spherix_world::chunk::biome::Biome;
use spherix_world::chunk::column::{ChunkColumnRef, ChunkColumnRefMut};
use spherix_world::chunk::heightmap::Heightmaps;
use spherix_world::chunk::palette::BlockGlobalPalette;
use spherix_world::chunk::status::ChunkStatus;
use spherix_world::chunk::vector::block::Vector2BlockSection;
use spherix_world::chunk::vector::Vector3BlockColumn;
use std::sync::Arc;

pub struct SurfaceMaterializer {
    palette: Arc<BlockGlobalPalette>
}

impl SurfaceMaterializer {
    pub fn new(palette: Arc<BlockGlobalPalette>) -> Self {
        Self {
            palette
        }
    }

    pub fn materialize(
        &self,
        noise_settings: &NoiseSettings,
        entropy_bag: Arc<EntropyBag>,
        rule_factory: Arc<RuleFactories>,
        noise_chunk: NoiseChunk,
        biome_accessor: BiomeAccessor,
        chunk_column: &mut ChunkColumn,
    ) {
        let mut col_accessor = ColumnAccessor { horizontal_pos: Vector2BlockSection::origin() };
        let biome_gradient = BiomeGradient::with_hashed_seed(
            1,
            &biome_accessor,
        );
        let cached_biome_gradient = LazyCachedBiomeGradient::new(&biome_gradient);

        let mut ctx = Context::new(
            WorldGenerationContext {
                height: noise_settings.noise_height as i32,
                min_y: noise_settings.noise_min_y,
            },
            unsafe {&*(&chunk_column.inner().heightmaps as *const Heightmaps)},
            SurfaceLevel::new(
                noise_chunk,
                noise_settings,
                entropy_bag.noises.surface.clone(),
                entropy_bag.rng.clone(),
            ),
            entropy_bag
        );

        let rule = rule_factory.create_rule(&mut ctx);

        let pos = chunk_column.pos();

        let chunk_min_x = pos.get_min_block_x();
        let chunk_min_z = pos.get_min_block_z();
        for local_x in 0..16u32 {
            for local_z in 0..16u32 {
                let x = chunk_min_x + local_x as i32;
                let z = chunk_min_z + local_z as i32;

                let height = chunk_column
                    .inner()
                    .heightmaps
                    .world_surface_wg
                    .as_ref()
                    .unwrap()
                    .height_section(Vector2BlockSection::new(local_x, local_z)) + 1;

                col_accessor.move_horizontally(Vector2BlockSection::new(local_x, local_z));

                let block_pos = Vector3::new(
                    x,
                    if noise_settings.use_legacy_random_source { 0 } else { height },
                    z
                );

                let biome = biome_gradient.biome(&block_pos);

                if biome.name() == "minecraft:eroded_badlands" {
                    self.eroded_badlands(&ctx, &col_accessor, chunk_column, Vector3::new(x, height, z))
                }

                let height = chunk_column
                    .inner()
                    .heightmaps
                    .world_surface_wg
                    .as_ref()
                    .unwrap()
                    .height_section(Vector2BlockSection::new(local_x, local_z)) + 1;

                ctx.update_xz(x, z);

                let mut stone_depth_above = 0;
                let mut water_height = i32::MIN;
                let mut stone_base_height = i32::MAX;
                let noise_min_y = noise_settings.noise_min_y;

                for y in (noise_min_y..=height).rev() {
                    let block_state = col_accessor.get_from(y, chunk_column);
                    let block = block_state.block();

                    if block.properties.is_air {
                        stone_depth_above = 0;
                        water_height = i32::MIN;
                    } else if block.properties.is_fluid {
                        if water_height == i32::MIN {
                            water_height = y + 1;
                        }
                    } else {
                        if stone_base_height >= y {
                            stone_base_height = -32512;

                            for y_i in (noise_min_y..y).rev() {
                                let block_state = col_accessor.get_from(y_i, chunk_column);
                                let block = block_state.block();

                                if block.properties.is_air || block.properties.is_fluid {
                                    stone_base_height = y_i + 1;
                                    break;
                                }
                            }
                        }

                        stone_depth_above += 1;
                        let stone_depth_below = y - stone_base_height + 1;

                        cached_biome_gradient.at(Vector3::new(x, y, z));
                        
                        ctx.update_y(stone_depth_above, stone_depth_below, water_height, y, &cached_biome_gradient);

                        if block == Block::STONE {
                            let block_state = rule.apply(Vector3::new(x, y, z), &mut ctx);
                            if block_state.is_some() {
                                col_accessor.set_to(y, block_state.unwrap(), chunk_column);
                            }
                        }
                    }
                }

                if biome.name() == "minecraft:frozen_ocean" || biome.name() == "minecraft:deep_frozen_ocean" {
                    self.frozen_ocean(
                        &mut ctx,
                        noise_settings.sea_level,
                        &col_accessor,
                        chunk_column,
                        biome,
                        Vector3::new(x, stone_depth_above, z),
                    )
                }
            }
        }

        chunk_column.inner_mut().status = ChunkStatus::Surface;
    }

    /// This method generates eroded pillar-like structures in the Badlands biome.
    fn eroded_badlands(&self, ctx: &Context, col_accessor: &ColumnAccessor, chunk_column: &mut ChunkColumn, at: Vector3) {
        // Noise "badlands_surface" generates a relatively smooth and broad pattern (it consists
        // of 3 octaves).
        let base_noise = abs(8.25 * ctx.entropy_bag.noises.badlands_surface.sample(Vector3f::new(at.x as f64, 0.0, at.z as f64)))
            // Noise "badlands_pillar" produces more detailed noise (because it consists of 4 octaves).
            // When multiplied by 15.0, it makes the volumes more like a narrow pillar shape.
            // This noise is scaled by the constant 0.2 for input variables, so it is less localized.
            .min(15.0 * ctx.entropy_bag.noises.badlands_pillar.sample(Vector3f::new(0.2 * at.x as f64, 0.0, 0.2 * at.z as f64)));

        // Proceed only if the height influence is greater than 0
        if base_noise <= 0.0 {
            return;
        }

        let default_state = self.palette.get_default_obj_by_index(&Block::STONE).unwrap();

        // Noise "badlands_pillar_roof" introduces variations in the height of the top of the terracotta pillars.
        let roof_noise = abs(1.5 * ctx.entropy_bag.noises.badlands_pillar_roof.sample(Vector3f::new(0.75 * at.x as f64, 0.0, 0.75 * at.z as f64)));
        // Large constant 64.0 makes these pillars much higher than another surface
        let final_height = 64.0 + (base_noise * base_noise * 2.5).min((roof_noise * 50.0).ceil() + 24.0);
        let final_height = floor(final_height);

        if at.y < final_height {
            // Scan downward from the calculated pillar top to find the ground. This loop prevents
            // the generation of pillars standing in water.
            for y in (-64..=final_height).rev() {
                let block_state = col_accessor.get_from(y, chunk_column);
                let block = block_state.block();
                // Stop scanning if solid ground is found
                if block == Block::STONE {
                    break
                }

                // We won't build pillar in water...
                if block == Block::WATER {
                    return;
                }
            }

            let mut y = final_height;
            // Fill the area with the default block from the pillar top down to the lowest point.
            // Default block will be replaced with appropriate block (most likely, with terracotta)
            // via Surface Rule.
            while y >= -64 && col_accessor.get_from(y, chunk_column).block().properties.is_air {
                col_accessor.set_to(y, default_state.clone(), chunk_column);
                y -= 1;
            }
        }
    }

    /// Generates ice structures (icebergs and pillars) in the frozen ocean biome. This function
    /// modifies a block column by adding packed ice and snow blocks, creating varied shapes
    /// based on multiple noise functions.
    fn frozen_ocean(
        &self,
        ctx: &mut Context,
        sea_level: i32,
        col_accessor: &ColumnAccessor,
        chunk_column: &mut ChunkColumn,
        biome: Arc<Biome>,
        at: Vector3
    ) {
        // Noise "iceberg_surface" generates a relatively smooth and broad pattern (it consists
        // of 3 octaves).
        let base_noise = abs(8.25 * ctx.entropy_bag.noises.iceberg_surface.sample(Vector3f::new(at.x as f64, 0.0, at.z as f64)))
            // Noise "iceberg_pillar" produces more detailed noise (because it consists of 4 octaves).
            // When multiplied by 15.0, it makes the iceberg more like a narrow pillar shape.
            // This noise is scaled by the constant 1.28 for input variables, so it is more localized.
            .min(15.0 * ctx.entropy_bag.noises.iceberg_pillar.sample(Vector3f::new(1.28 * at.x as f64, 0.0, 1.28 * at.z as f64)));

        // If the calculated height is not big enough, return without generating any structure
        if base_noise <= 1.8 {
            return;
        }

        // Noise "roof_noise" introduces variations in the height of the top of the ice pillars.
        let roof_noise = abs(1.5 * ctx.entropy_bag.noises.iceberg_pillar_roof.sample(
            Vector3f::new(1.17 * at.x as f64, 0.0, 1.17 * at.z as f64))
        );
        let mut final_height = (base_noise * base_noise * 1.2).min((roof_noise * 40.0).ceil() + 14.0);
        if should_melt_frozen_ocean_iceberg_slightly(&Vector3::new(at.x, 0, at.z), biome.as_ref()) {
            // Slightly decrease final height if temperature high enough
            final_height -= 2.0;
        }

        // Determine the lower bound of where to replace water blocks with ice below sea level
        // during the iceberg generation process.
        let iceberg_lower_bound;
        // This condition acts as a threshold, determining whether we should attempt to
        // generate the underwater portion of the iceberg.
        // So it essentially decides whether the iceberg is tall enough to warrant extending
        // below sea level.
        if final_height > 2.0 {
            iceberg_lower_bound = sea_level as f64 - final_height - 7.0;
            final_height += sea_level as f64;
        } else {
            final_height = 0.0;
            iceberg_lower_bound = 0.0;
        }

        let mut rng = ctx.entropy_bag.rng.at(Vector3::new(at.x, 0, at.z));
        // This line suggests that the height of snow block pillar can not be less than 2.
        // But practically it can be restricted by min_surface_level().
        let total_snow_blocks = 2 + rng.next_u32(4);
        // Calculates the starting height for snow placement on the ice structure.
        // It's 18 blocks above sea level plus a random value between 0 and 9.
        let snow_blocks_start_height = sea_level + 18 + rng.next_u32(10) as i32;
        let mut snow_blocks = 0;

        let snow_block = self.palette.get_default_obj_by_index(&Block::SNOW_BLOCK).unwrap();
        let packed_ice = self.palette.get_default_obj_by_index(&Block::PACKED_ICE).unwrap();

        let mut y = at.y.max(final_height as i32 + 1);

        // Loop through the block column from the current height down to the estimated min_surface_level()
        while y >= ctx.surface_level.min_surface_level() {
            let block_state = col_accessor.get_from(y, chunk_column);
            let block = block_state.block();

            // Checks if the block is air inside the iceberg height or if it's water below the
            // sea level.
            if (
                block.properties.is_air
                && y < final_height as i32
                // Random condition adds some erosion for above-water ice/snow volumes
                && rng.next_f64() > 0.01
            ) || (
                block.properties.material() == &Material::WATER
                && y > iceberg_lower_bound as i32
                && y < sea_level
                && iceberg_lower_bound != 0.0
                // Random condition adds some erosion for underwater ice volumes. So this
                // randomization forms empty (non-iced) blocks below sea level. Note that
                // the chance of such outcome is much higher than the same chance for
                // above water iceberg part.
                && rng.next_f64() > 0.15
            ) {
                // Set snow block for above-water iceberg if this high enough
                if snow_blocks <= total_snow_blocks && y > snow_blocks_start_height {
                    col_accessor.set_to(y, snow_block.clone(), chunk_column);
                    snow_blocks += 1
                } else {
                    col_accessor.set_to(y, packed_ice.clone(), chunk_column);
                }
            }

            y -= 1;
        }
    }
}

struct ColumnAccessor {
    horizontal_pos: Vector2BlockSection
}

impl ColumnAccessor {
    #[inline]
    fn move_horizontally(&mut self, at: Vector2BlockSection) {
        self.horizontal_pos = at;
    }

    #[inline]
    fn get_from(&self, y: i32, from: &ChunkColumn) -> Arc<BlockState> {
        unsafe {
            from.with_unsafe().block_state(Vector3BlockColumn::new(self.horizontal_pos.x(), y, self.horizontal_pos.z()))
        }
    }

    #[inline]
    fn set_to(&self, y: i32, block: Arc<BlockState>, to: &mut ChunkColumn) {
        unsafe {
            to.with_unsafe_mut().set_block_state(Vector3BlockColumn::new(self.horizontal_pos.x(), y, self.horizontal_pos.z()), block);
        }
    }
}
