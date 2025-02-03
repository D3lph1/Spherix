use crate::aquifer::Aquifer;
use crate::noise::density::cache::CacheAllInCell;
use crate::noise::density::density::{ContextFiller, DensityFunction, DensityFunctionContext, DensityFunctions};
use crate::noise::density::maker::InterpolatedInner;
use crate::noise::router::NoiseRouter;
use spherix_math::vector::Vector3;
use spherix_world::block::state::BlockState;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

pub struct NoiseChunk {
    pub ctx: DensityFunctionContext,
    aquifer: Box<dyn Aquifer>,
    interpolators: Vec<Rc<RefCell<InterpolatedInner>>>,
    cell_caches: Vec<Rc<RefCell<CacheAllInCell>>>,
    pub df: DensityFunctions,
}

impl NoiseChunk {
    pub fn new(
        ctx: DensityFunctionContext,
        router: NoiseRouter,
        aquifer: Box<dyn Aquifer>,
        interpolators: Vec<Rc<RefCell<InterpolatedInner>>>,
    ) -> Self {
        let cache_cell = Rc::new(
            RefCell::new(
                CacheAllInCell::new(router.final_density, ctx.cell_width, ctx.cell_height)
            )
        );

        let mut chunk = Self {
            ctx,
            aquifer,
            interpolators,
            cell_caches: vec![cache_cell.clone()],
            df: DensityFunctions::CacheAllInCell(cache_cell),
        };

        chunk.initialize_for_first_cell_x();

        chunk
    }
    
    fn vector(&self) -> Vector3 {
        Vector3::new(
            self.ctx.cell_start_block_x + self.ctx.in_cell_x as i32,
            self.ctx.cell_start_block_y + self.ctx.in_cell_y as i32,
            self.ctx.cell_start_block_z + self.ctx.in_cell_z as i32,
        )
    }

    pub fn calculate_interpolated_state(&mut self) -> Option<Arc<BlockState>> {
        // self.ctx.debug_tree.reset();

        let sampled = self.df.sample(
            self.vector(),
            &mut self.ctx,
        );

        // println!("{}", self.ctx.debug_tree);

        self.aquifer.compute(&self.ctx, sampled)
    }

    fn initialize_for_first_cell_x(&mut self) {
        self.ctx.interpolating = true;
        self.fill_slice(true, self.ctx.first_cell_x);
    }

    fn fill_slice(&mut self, use_first_slice: bool, start_coord: i32) {
        self.ctx.cell_start_block_x = start_coord * self.ctx.cell_width as i32;
        self.ctx.in_cell_x = 0;

        for i in 0..=self.ctx.cell_count_xz as i32 {
            let j = self.ctx.first_cell_z + i;
            self.ctx.cell_start_block_z = j * self.ctx.cell_width as i32;
            self.ctx.in_cell_z = 0;
            self.ctx.array_interpolation_counter += 1;

            for interpolator in self.interpolators.iter_mut() {
                let mut interpolator = interpolator.borrow_mut();

                let prev_filler = self.ctx.filler.clone();
                self.ctx.filler = ContextFiller::Slice;

                if use_first_slice {
                    interpolator.fill_slice0(i as usize, &mut self.ctx)
                } else {
                    interpolator.fill_slice1(i as usize, &mut self.ctx)
                }

                self.ctx.filler = prev_filler;
            }
        }

        self.ctx.array_interpolation_counter += 1;
    }

    pub fn advance_cell_x(&mut self, x: i32) {
        self.fill_slice(false, self.ctx.first_cell_x + x + 1);
        self.ctx.cell_start_block_x = (self.ctx.first_cell_x + x) * self.ctx.cell_width as i32;
    }

    pub fn select_cell_yz(&mut self, y: i32, z: i32) {
        self.interpolators.iter_mut().for_each(|interpolator| {
            interpolator.borrow_mut().select_cell_yz(y as usize, z as usize)
        });

        self.ctx.filling_cell = true;
        self.ctx.cell_start_block_y = (y + self.ctx.cell_noise_min_y) * self.ctx.cell_height as i32;
        self.ctx.cell_start_block_z = (self.ctx.first_cell_z + z) * self.ctx.cell_width as i32;
        self.ctx.array_interpolation_counter += 1;

        for cell_cache in self.cell_caches.iter() {
            cell_cache.borrow_mut().fill_array_inner(&mut self.ctx);
        }

        self.ctx.array_interpolation_counter += 1;
        self.ctx.filling_cell = false;
    }

    pub fn update_for_y(&mut self, y: i32, interpolated: f64) {
        self.ctx.in_cell_y = (y - self.ctx.cell_start_block_y) as u32;
        self.interpolators.iter_mut().for_each(|interpolator| {
            interpolator.borrow_mut().update_for_y(interpolated);
        });
    }

    pub fn update_for_x(&mut self, x: i32, interpolated: f64) {
        self.ctx.in_cell_x = (x - self.ctx.cell_start_block_x) as u32;
        self.interpolators.iter_mut().for_each(|interpolator| {
            interpolator.borrow_mut().update_for_x(interpolated);
        });
    }

    pub fn update_for_z(&mut self, z: i32, interpolated: f64) {
        self.ctx.in_cell_z = (z - self.ctx.cell_start_block_z) as u32;
        self.ctx.interpolation_counter += 1;
        self.interpolators.iter_mut().for_each(|interpolator| {
            interpolator.borrow_mut().update_for_z(interpolated);
        });
    }

    pub fn swap_slices(&mut self) {
        self.interpolators
            .iter()
            .for_each(|interpolator| interpolator.borrow_mut().swap_slices())
    }

    pub fn stop_interpolation(&mut self) {
        self.ctx.interpolating = false;
    }
}
