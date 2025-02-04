use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use spherix_math::vector::Vector3;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct BlendAlpha;

impl Debug for BlendAlpha {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlendAlpha")
    }
}

impl DensityFunction for BlendAlpha {
    fn sample(&self, _: Vector3, _: &mut DensityFunctionContext) -> f64 {
        // ctx.get_or_compute_blending_output(pos.x, pos.z).alpha
        1.0
    }

    fn fill_array(&self, arr: &mut [f64], _: &mut DensityFunctionContext) {
        for i in 0..arr.len() {
            arr[i] = 1.0
        }
    }

    fn min_value(&self) -> f64 {
        1.0
    }

    fn max_value(&self) -> f64 {
        1.0
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::BlendAlpha(self)
        )
    }
}

#[derive(Clone)]
pub struct BlendOffset;

impl Debug for BlendOffset {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "BlendOffset")
    }
}

impl DensityFunction for BlendOffset {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        ctx.get_or_compute_blending_output(at.x, at.z).offset
    }

    fn min_value(&self) -> f64 {
        0.0
    }

    fn max_value(&self) -> f64 {
        0.0
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::BlendOffset(self)
        )
    }
}
