use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use debug_tree::add_branch_to;
use spherix_math::vector::Vector3;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct DebugDensityFunction(DensityFunctions);

impl DebugDensityFunction {
    #[inline]
    pub fn new(inner: DensityFunctions) -> Self {
        Self(inner)
    }
}

impl Debug for DebugDensityFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Debug")
    }
}

impl DensityFunction for DebugDensityFunction {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let tree = ctx.debug_tree.as_mut().unwrap();
        
        add_branch_to!(tree.tree, "{:?}: %{}%", self.0, tree.counter);
        let counter = tree.counter;
        tree.counter += 1;

        let sampled = self.0.sample(at, ctx);
        let tree = ctx.debug_tree.as_mut().unwrap();
        tree.map.insert(counter, sampled);

        sampled
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.0.fill_array(arr, ctx);
    }

    fn min_value(&self) -> f64 {
        self.0.min_value()
    }

    fn max_value(&self) -> f64 {
        self.0.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        self.0.map(mapper)
    }
}
