use crate::chunk::noise::NoiseChunk;
use crate::noise::density::cache::{quart_pos_from_block, quart_pos_to_block};
use crate::noise::density::density::DensityFunction;
use crate::noise::density::noise::NoiseHolder;
use crate::noise::math::{floor, lerp2};
use crate::noise::perlin::{DefaultNoise, Noise};
use crate::noise::settings::NoiseSettings;
use crate::rng::{Rng, RngPos, XoroShiroPos};
use gxhash::GxBuildHasher;
use spherix_math::vector::{Vector2, Vector3, Vector3f};
use spherix_world::chunk::pos::ChunkPos;
use std::collections::HashMap;
use std::sync::Arc;

pub struct SurfaceLevel<'a> {
    noise_chunk: NoiseChunk,
    noise_settings: &'a NoiseSettings,
    pub block: Vector3,
    cell_height: usize,
    pub water_height: i32,
    pub stone_depth_below: i32,
    pub stone_depth_above: i32,
    last_update_xz: i64, // -9223372036854775807
    last_update_y: i64,
    last_preliminary_surface_cell_origin: i64,
    preliminary_surface_cache: [i32; 4],
    surface_secondary: f64,
    last_min_surface_level_update: i64,
    min_surface_level: i32,
    pub surface_depth: i32,
    last_surface_depth2update: i64,
    surface_noise: Arc<NoiseHolder<DefaultNoise>>,
    preliminary_surface_level_cache: HashMap<i64, i32, GxBuildHasher>,
    noise_rng: Arc<XoroShiroPos>,
}

impl<'a> SurfaceLevel<'a> {
    const DENSITY_THRESHOLD: f64 = 0.390625;

    const HOW_FAR_BELOW_PRELIMINARY_SURFACE_LEVEL_TO_BUILD_SURFACE: i32 = 8;
    const SURFACE_CELL_BITS: i32 = 4;
    const SURFACE_CELL_SIZE: f32 = 16.0;
    const SURFACE_CELL_MASK: i32 = 15;

    pub fn new(noise_chunk: NoiseChunk, noise_settings: &'a NoiseSettings, surface_noise: Arc<NoiseHolder<DefaultNoise>>, noise_rng: Arc<XoroShiroPos>) -> Self {
        Self {
            noise_chunk,
            noise_settings,
            block: Vector3::origin(),
            cell_height: noise_settings.cell_height() as usize,
            water_height: 0,
            stone_depth_below: 0,
            stone_depth_above: 0,
            last_update_xz: -0x7FFFFFFFFFFFFFFF,
            last_update_y: -0x7FFFFFFFFFFFFFFF,
            last_preliminary_surface_cell_origin: 0,
            preliminary_surface_cache: [0, 0, 0, 0],
            surface_secondary: 0.0,
            last_min_surface_level_update: -0x7FFFFFFFFFFFFFFF - 1,
            min_surface_level: 0,
            surface_depth: 0,
            last_surface_depth2update: -0x7FFFFFFFFFFFFFFF - 1,
            surface_noise,
            preliminary_surface_level_cache: Default::default(),
            noise_rng,
        }
    }

    pub fn update_xz(&mut self, x: i32, z: i32) {
        self.last_update_xz += 1;
        self.last_update_y += 1;
        self.block = Vector3::new(x, self.block.y(), z);
        self.surface_depth = self.surface_depth(x, z);
    }

    pub fn update_y(&mut self, y: i32, water_height: i32, stone_depth_below: i32, stone_depth_above: i32) {
        self.last_update_y += 1;
        self.block = Vector3::new(self.block.x(), y, self.block.z());
        self.water_height = water_height;
        self.stone_depth_below = stone_depth_below;
        self.stone_depth_above = stone_depth_above;
    }

    fn surface_depth(&self, x: i32, z: i32) -> i32 {
        let noise = self.surface_noise.sample(Vector3f::new(x as f64, 0.0, z as f64));

        (noise * 2.75 + 3.0 + self.noise_rng.at(Vector3::new(x, 0, z)).next_f64() * 0.25) as i32
    }

    pub fn min_surface_level(&mut self) -> i32 {
        if self.last_min_surface_level_update != self.last_update_xz {
            self.last_min_surface_level_update = self.last_update_xz;
            let surface_cell_x = Self::block_coord_to_surface_cell(self.block.x());
            let surface_cell_z = Self::block_coord_to_surface_cell(self.block.z());
            let k: i64 = Vector2::new(surface_cell_x, surface_cell_z).into();

            if self.last_preliminary_surface_cell_origin != k {
                self.last_preliminary_surface_cell_origin = k;
                self.preliminary_surface_cache[0] = self.preliminary_surface_level(
                    Self::surface_cell_to_block_coord(surface_cell_x),
                    Self::surface_cell_to_block_coord(surface_cell_z)
                );
                self.preliminary_surface_cache[1] = self.preliminary_surface_level(
                    Self::surface_cell_to_block_coord(surface_cell_x + 1),
                    Self::surface_cell_to_block_coord(surface_cell_z)
                );
                self.preliminary_surface_cache[2] = self.preliminary_surface_level(
                    Self::surface_cell_to_block_coord(surface_cell_x),
                    Self::surface_cell_to_block_coord(surface_cell_z + 1)
                );
                self.preliminary_surface_cache[3] = self.preliminary_surface_level(
                    Self::surface_cell_to_block_coord(surface_cell_x + 1),
                    Self::surface_cell_to_block_coord(surface_cell_z + 1)
                );
            }

            let interpolated = floor(lerp2(
                ((self.block.x() & Self::SURFACE_CELL_MASK) as f32 / Self::SURFACE_CELL_SIZE) as f64,
                ((self.block.z() & Self::SURFACE_CELL_MASK) as f32 / Self::SURFACE_CELL_SIZE) as f64,
                self.preliminary_surface_cache[0] as f64,
                self.preliminary_surface_cache[1] as f64,
                self.preliminary_surface_cache[2] as f64,
                self.preliminary_surface_cache[3] as f64,
            ));
            self.min_surface_level = interpolated + self.surface_depth - Self::HOW_FAR_BELOW_PRELIMINARY_SURFACE_LEVEL_TO_BUILD_SURFACE;
        }

        self.min_surface_level
    }

    fn preliminary_surface_level(&mut self, x: i32, z: i32) -> i32 {
        let x1 = quart_pos_to_block(quart_pos_from_block(x));
        let z1 = quart_pos_to_block(quart_pos_from_block(z));
        let coord_as_i64 = Vector2::new(x1, z1).into();

        // if coord_as_i64 == 755914244208 {
        //     println!("!")
        // }

        if self
            .preliminary_surface_level_cache
            .contains_key(&coord_as_i64)
        {
            *self
                .preliminary_surface_level_cache
                .get(&coord_as_i64)
                .unwrap()
        } else {
            let preliminary_surface_level = self.compute_preliminary_surface(coord_as_i64);
            self.preliminary_surface_level_cache
                .insert(coord_as_i64, preliminary_surface_level);

            preliminary_surface_level
        }
    }

    fn compute_preliminary_surface(&mut self, coord_as_i64: i64) -> i32 {
        let x = ChunkPos::extract_x(coord_as_i64);
        let z = ChunkPos::extract_z(coord_as_i64);
        let min_y = self.noise_settings.noise_min_y;
        let height = self.noise_settings.noise_height as i32;

        for y in (min_y..=min_y + height).rev().step_by(self.cell_height) {
            let at = Vector3::new(x, y, z);
            let sampled = self
                .noise_settings
                .router
                .initial_density_without_jaggedness
                .sample(at, &mut self.noise_chunk.ctx);

            if sampled > Self::DENSITY_THRESHOLD {
                return y;
            }
        }

        i32::MAX
    }

    pub fn surface_secondary(&mut self) -> f64 {
        if self.last_surface_depth2update != self.last_update_xz {
            self.last_surface_depth2update = self.last_update_xz;
            self.surface_secondary = self.surface_noise.sample(Vector3f::new(
                self.block.x() as f64,
                0.0,
                self.block.z() as f64,
            ));
        }

        self.surface_secondary
    }

    fn block_coord_to_surface_cell(coord: i32) -> i32 {
        coord >> Self::SURFACE_CELL_BITS
    }

    fn surface_cell_to_block_coord(cell: i32) -> i32 {
        cell << Self::SURFACE_CELL_BITS
    }
}
