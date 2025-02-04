use crate::noise::density::density::{fill_all_directly, ContextFiller, DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use crate::noise::math::{lerp, lerp3};
use spherix_math::vector::Vector3;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::mem;
use std::rc::Rc;

#[derive(Clone)]
pub struct InterpolatedInner {
    pub argument: DensityFunctions,
    pub cell_count_y: usize,
    pub cell_count_xz: usize,
    pub slice0: Vec<Vec<f64>>,
    slice1: Vec<Vec<f64>>,
    noise000: f64,
    noise001: f64,
    noise100: f64,
    noise101: f64,
    noise010: f64,
    noise011: f64,
    noise110: f64,
    noise111: f64,
    value_xz00: f64,
    value_xz10: f64,
    value_xz01: f64,
    value_xz11: f64,
    value_z0: f64,
    value_z1: f64,
    value: f64,
}

impl InterpolatedInner {
    pub fn new(argument: DensityFunctions, cell_count_y: usize, cell_count_xz: usize) -> Self {
        Self {
            argument,
            cell_count_y,
            cell_count_xz,
            slice0: Self::allocate_slice(cell_count_y, cell_count_xz),
            slice1: Self::allocate_slice(cell_count_y, cell_count_xz),
            noise000: 0.0,
            noise001: 0.0,
            noise100: 0.0,
            noise101: 0.0,
            noise010: 0.0,
            noise011: 0.0,
            noise110: 0.0,
            noise111: 0.0,
            value_xz00: 0.0,
            value_xz10: 0.0,
            value_xz01: 0.0,
            value_xz11: 0.0,
            value_z0: 0.0,
            value_z1: 0.0,
            value: 0.0,
        }
    }

    fn allocate_slice(cell_count_y: usize, cell_count_xz: usize) -> Vec<Vec<f64>> {
        let i = cell_count_xz + 1;
        let j = cell_count_y + 1;
        vec![vec![0f64; j]; i]
    }

    pub fn select_cell_yz(&mut self, y: usize, z: usize) {
        self.noise000 = self.slice0[z][y];
        self.noise001 = self.slice0[z + 1][y];
        self.noise100 = self.slice1[z][y];
        self.noise101 = self.slice1[z + 1][y];
        self.noise010 = self.slice0[z][y + 1];
        self.noise011 = self.slice0[z + 1][y + 1];
        self.noise110 = self.slice1[z][y + 1];
        self.noise111 = self.slice1[z + 1][y + 1];
    }

    pub fn update_for_y(&mut self, y: f64) {
        self.value_xz00 = lerp(y, self.noise000, self.noise010);
        self.value_xz10 = lerp(y, self.noise100, self.noise110);
        self.value_xz01 = lerp(y, self.noise001, self.noise011);
        self.value_xz11 = lerp(y, self.noise101, self.noise111);
    }

    pub fn update_for_x(&mut self, x: f64) {
        self.value_z0 = lerp(x, self.value_xz00, self.value_xz10);
        self.value_z1 = lerp(x, self.value_xz01, self.value_xz11);
    }

    pub fn update_for_z(&mut self, z: f64) {
        self.value = lerp(z, self.value_z0, self.value_z1);
    }

    pub fn slice0_mut(&mut self) -> &mut Vec<Vec<f64>> {
        &mut self.slice0
    }

    pub fn slice1_mut(&mut self) -> &mut Vec<Vec<f64>> {
        &mut self.slice1
    }

    pub fn fill_slice0(&mut self, index: usize, ctx: &mut DensityFunctionContext) {
        if ctx.filling_cell {
            // This macro helps to bypass borrow checker restrictions on having
            // shared and mutable borrows simultaneously
            fill_all_directly!(ctx, self, self.slice0[index]);
        } else {
            self.argument.fill_array(&mut self.slice0[index], ctx);
        }
    }

    pub fn fill_slice1(&mut self, index: usize, ctx: &mut DensityFunctionContext) {
        if ctx.filling_cell {
            // This macro helps to bypass borrow checker restrictions on having
            // shared and mutable borrows simultaneously
            fill_all_directly!(ctx, self, self.slice1[index]);
        } else {
            self.argument.fill_array(&mut self.slice1[index], ctx);
        }
    }

    pub fn swap_slices(&mut self) {
        mem::swap(&mut self.slice0, &mut self.slice1);
    }
}

impl Debug for InterpolatedInner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            concat!(
            "InterpolatedInner (noise000: {}, noise001: {}, noise100: {}, noise101: {}, ",
            "noise010: {}, noise011: {}, noise110: {}, noise111: {})"
            ),
            self.noise000,
            self.noise001,
            self.noise100,
            self.noise101,
            self.noise010,
            self.noise011,
            self.noise110,
            self.noise111
        )
    }
}

impl DensityFunction for InterpolatedInner {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        if ctx.filler != ContextFiller::Default {
            self.argument.sample(at, ctx)
        } else {
            if ctx.filling_cell {
                lerp3(
                    ctx.in_cell_x as f64 / ctx.cell_width as f64,
                    ctx.in_cell_y as f64 / ctx.cell_height as f64,
                    ctx.in_cell_z as f64 / ctx.cell_width as f64,
                    self.noise000, self.noise100,
                    self.noise010, self.noise110,
                    self.noise001, self.noise101,
                    self.noise011, self.noise111,
                )
            } else {
                self.value
            }
        }
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        if ctx.filling_cell {
            ctx.fill_all_directly(arr, self);
        } else {
            self.argument.fill_array(arr, ctx);
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
            DensityFunctions::InterpolatedInner(
                Rc::new(
                    RefCell::new(
                        InterpolatedInner {
                            argument: self.argument.map(mapper),
                            cell_count_y: self.cell_count_y,
                            cell_count_xz: self.cell_count_xz,
                            slice0: self.slice0,
                            slice1: self.slice1,
                            noise000: self.noise000,
                            noise001: self.noise001,
                            noise100: self.noise100,
                            noise101: self.noise101,
                            noise010: self.noise010,
                            noise011: self.noise011,
                            noise110: self.noise110,
                            noise111: self.noise111,
                            value_xz00: self.value_xz00,
                            value_xz10: self.value_xz10,
                            value_xz01: self.value_xz01,
                            value_xz11: self.value_xz11,
                            value_z0: self.value_z0,
                            value_z1: self.value_z1,
                            value: self.value,
                        }
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Interpolated(DensityFunctions);

impl Interpolated {
    pub fn new(argument: DensityFunctions, cell_count_y: usize, cell_count_xz: usize) -> Self {
        Self(
            DensityFunctions::InterpolatedInner(
                Rc::new(
                    RefCell::new(
                        InterpolatedInner::new(argument, cell_count_y, cell_count_xz)
                    )
                )
            )
        )
    }
}

impl Debug for Interpolated {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Interpolated (...)")
    }
}

impl DensityFunction for Interpolated {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.0.sample(at, ctx)
    }

    fn min_value(&self) -> f64 {
        self.0.min_value()
    }

    fn max_value(&self) -> f64 {
        self.0.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Interpolated(Box::new(Self(self.0.map(mapper))))
        )
    }
}

#[derive(Clone)]
pub struct Marker {
    ty: String,
    inner: DensityFunctions
}

impl Marker {
    pub fn new(ty: String, inner: DensityFunctions) -> Self {
        Self {
            ty,
            inner,
        }
    }
}

impl Debug for Marker {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ty)
    }
}

impl DensityFunction for Marker {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.inner.sample(at, ctx)
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
        mapper.map(DensityFunctions::Marker(Box::new(Marker::new(self.ty, self.inner.map(mapper)))))
    }
}
