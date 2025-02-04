use crate::noise::density::density::{ContextFiller, DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use spherix_math::vector::Vector3;
use spherix_util::slice::slice_copy;
use spherix_world::chunk::pos::ChunkPos;
use std::cell::{Cell, RefCell};
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct FlatCache {
    pub argument: DensityFunctions,
    values: Vec<Vec<f64>>,
    empty: bool
}

impl FlatCache {
    pub fn new(
        argument: DensityFunctions,
        ctx: &mut DensityFunctionContext,
        fill: bool,
    ) -> Self {
        let noise_size_xz = ctx.noise_size_xz as usize;
        let mut values = vec![vec![0f64; noise_size_xz + 1]; noise_size_xz + 1];
        if fill {
            for i in 0..=noise_size_xz {
                let j = ctx.first_noise_x + i as i32;
                let k = to_block_pos(j);

                for l in 0..=noise_size_xz {
                    let i1 = ctx.first_noise_z + l as i32;
                    let j1 = to_block_pos(i1);

                    let sampled = argument.sample(Vector3::new(k, 0, j1), ctx);

                    values[i][l] = sampled;
                }
            }
        }

        Self {
            argument,
            values,
            empty: true
        }
    }
}

impl Debug for FlatCache {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FlatCache ()")
    }
}

impl DensityFunction for FlatCache {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        if self.empty {
            return self.argument.sample(at, ctx)
        }

        let i = from_block_pos(at.x);
        let j = from_block_pos(at.z);
        let k = i - ctx.first_noise_x;
        let l = j - ctx.first_noise_z;
        let i1 = self.values.len() as i32;

        if k >= 0 && l >= 0 && k < i1 && l < i1 {
            self.values[k as usize][l as usize]
        } else {
            self.argument.sample(at, ctx)
        }
    }

    fn min_value(&self) -> f64 {
        self.argument.min_value()
    }

    fn max_value(&self) -> f64 {
        self.argument.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::FlatCache(
                Box::new(
                    Self {
                        argument: self.argument.map(mapper),
                        values: self.values,
                        empty: self.empty
                    }
                )
            )
        )
    }
}

#[inline]
fn to_block_pos(pos: i32) -> i32 {
    pos << 2
}

#[inline]
pub const fn from_block_pos(pos: i32) -> i32 {
    pos >> 2
}

#[inline]
pub fn block_to_section_coord(block: i32) -> i32 {
    block >> 4
}

#[inline]
pub fn section_to_block_coord(section: i32) -> i32 {
    section << 4
}

#[inline]
pub const fn quart_pos_from_section(section: i32) -> i32 {
    section << 2
}

#[inline]
pub const fn quart_pos_to_section(quart_pos: i32) -> i32 {
    quart_pos >> 2
}

#[inline]
pub fn quart_pos_from_block(block: i32) -> i32 {
    block >> 2
}



#[inline]
pub fn quart_pos_to_block(block: i32) -> i32 {
    block << 2
}

#[inline]
fn length_squared_2(x: f64, y: f64) -> f64 {
    x * x + y * y
}

#[inline]
fn length_squared(x: f64, y: f64, z: f64) -> f64 {
    x * x + y * y + z * z
}

#[inline]
pub fn length_2(x: f64, y: f64) -> f64 {
    length_squared_2(x, y).sqrt()
}

#[inline]
pub fn length(x: f64, y: f64, z: f64) -> f64 {
    length_squared(x, y, z).sqrt()
}

#[derive(Clone)]
pub struct Cache2D {
    inner: DensityFunctions,
    last_pos_2d: Cell<i64>,
    last_value: Cell<f64>,
    empty: bool
}

impl Cache2D {
    pub fn new(inner: DensityFunctions) -> Self {
        Self {
            inner,
            last_pos_2d: Cell::new(ChunkPos::INVALID.into()),
            last_value: Cell::new(0.0),
            empty: true
        }
    }
}

impl Debug for Cache2D {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cache2D (last_pos_2d: {}, last_value: {})", self.last_pos_2d.get(), self.last_value.get())
    }
}

impl DensityFunction for Cache2D {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        if self.empty {
            return self.inner.sample(at, ctx);
        }

        let i = at.x;
        let j = at.z;
        let k = ChunkPos::new(i, j).into();
        if self.last_pos_2d.get() == k {
            self.last_value.get()
        } else {
            self.last_pos_2d.set(k);
            let value = self.inner.sample(at, ctx);
            self.last_value.set(value);
            value
        }
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.inner.fill_array(arr, ctx)
    }

    fn min_value(&self) -> f64 {
        self.inner.min_value()
    }

    fn max_value(&self) -> f64 {
        self.inner.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Cache2D(
                Box::new(
                    Self::new(
                        self.inner.map(mapper)
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct CacheOnce {
    inner: DensityFunctions,
    last_counter: Cell<usize>,
    last_array_counter: Cell<usize>,
    last_value: Cell<f64>,
    last_array: RefCell<Option<Vec<f64>>>,
}

impl CacheOnce {
    pub fn new(inner: DensityFunctions) -> Self {
        Self {
            inner,
            last_counter: Cell::new(0),
            last_array_counter: Cell::new(0),
            last_value: Cell::new(0.0),
            last_array: RefCell::new(None),
        }
    }
}

impl Debug for CacheOnce {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheOnce")
    }
}

impl DensityFunction for CacheOnce {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        if ctx.filler != ContextFiller::Default {
            self.inner.sample(at, ctx )
        } else if self.last_array.borrow().is_some() && self.last_array_counter.get() == ctx.array_interpolation_counter {
            self.last_array.borrow().as_ref().unwrap()[ctx.array_index]
        } else if self.last_counter.get() == ctx.interpolation_counter {
            self.last_value.get()
        } else {
            self.last_counter.set(ctx.interpolation_counter);
            let d0 = self.inner.sample(at, ctx);
            self.last_value.set(d0);

            d0
        }
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        if self.last_array.borrow().is_some() && self.last_array_counter.get() == ctx.array_interpolation_counter {
            slice_copy(self.last_array.borrow().as_ref().unwrap(), 0, arr, 0, arr.len());
        } else {
            self.inner.fill_array(arr, ctx);
            if self.last_array.borrow().is_some() && self.last_array.borrow().as_ref().unwrap().len() == arr.len() {
                slice_copy(arr, 0, self.last_array.borrow_mut().as_mut().unwrap(), 0, arr.len());
            } else {
                self.last_array.replace(Some(arr.into()));
            }

            self.last_array_counter.set(ctx.array_interpolation_counter)
        }
    }

    fn min_value(&self) -> f64 {
        self.inner.min_value()
    }

    fn max_value(&self) -> f64 {
        self.inner.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::CacheOnce(
                Box::new(
                    Self::new(
                        self.inner.map(mapper)
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct CacheAllInCell {
    inner: DensityFunctions,
    values: Vec<f64>,
}

impl CacheAllInCell {
    pub fn new(inner: DensityFunctions, cell_width: u32, cell_height: u32) -> Self {
        Self {
            inner,
            values: vec![0.0; (cell_width * cell_width * cell_height) as usize],
        }
    }

    pub fn fill_array_inner(&mut self, ctx: &mut DensityFunctionContext) {
        self.inner.fill_array(&mut self.values, ctx);
    }
}

impl Debug for CacheAllInCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheAllInCell")
    }
}

impl DensityFunction for CacheAllInCell {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        if ctx.filler != ContextFiller::Default {
            return self.inner.sample(at, ctx)
        }

        let i = ctx.in_cell_x;
        let j = ctx.in_cell_y;
        let k = ctx.in_cell_z;

        if i >= 0 && j >= 0 && k >= 0 && i < ctx.cell_width && j < ctx.cell_height && k < ctx.cell_width {
            self.values[(((ctx.cell_height - 1 - j) * ctx.cell_width + i) * ctx.cell_width + k) as usize]
        } else {
            self.inner.sample(at, ctx)
        }
    }

    fn min_value(&self) -> f64 {
        self.inner.min_value()
    }

    fn max_value(&self) -> f64 {
        self.inner.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        todo!()
    }
}
