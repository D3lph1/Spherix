use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use spherix_math::vector::Vector3;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct AddConst {
    input: DensityFunctions,
    argument: f64,
    min_value: f64,
    max_value: f64,
}

impl AddConst {
    pub fn new(input: DensityFunctions, argument: f64) -> Self {
        let min_value = input.min_value() + argument;
        let max_value = input.max_value() + argument;

        Self {
            input,
            argument,
            min_value,
            max_value,
        }
    }
}

impl Debug for AddConst {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "AddConst (argument: {}, min_value: {}, max_value: {})", self.argument, self.min_value, self.max_value)
    }
}

impl DensityFunction for AddConst {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.input.sample(at, ctx) + self.argument
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.input.fill_array(arr, ctx);

        for i in 0..arr.len() {
            arr[i] = arr[i] + self.argument;
        }
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::AddConst(
                Box::new(
                    Self::new(self.input.map(mapper), self.argument)
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Add {
    argument1: DensityFunctions,
    argument2: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Add {
    pub fn new(argument1: DensityFunctions, argument2: DensityFunctions) -> Self {
        let d0 = argument1.min_value();
        let d1 = argument2.min_value();
        let d2 = argument1.max_value();
        let d3 = argument2.max_value();

        let min_value = d0 + d1;
        let max_value = d2 + d3;

        Self {
            argument1,
            argument2,
            min_value,
            max_value,
        }
    }
}

impl Debug for Add {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Add (min_value: {}. max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Add {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let x = self.argument1.sample(at, ctx);
        let y = self.argument2.sample(at, ctx);

        x + y
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.argument1.fill_array(arr, ctx);

        let mut new_arr = vec![0.0; arr.len()];
        self.argument2.fill_array(&mut new_arr, ctx);

        for k in 0..arr.len() {
            arr[k] += new_arr[k];
        }
    }

    #[inline]
    fn min_value(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Add(
                Box::new(
                    Add::new(
                        self.argument1.map(mapper),
                        self.argument2.map(mapper),
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct MulConst {
    input: DensityFunctions,
    argument: f64,
    min_value: f64,
    max_value: f64,
}

impl MulConst {
    pub fn new(input: DensityFunctions, argument: f64) -> Self {
        let d0 = input.min_value();
        let d1 = argument;
        let d2 = input.max_value();
        let d3 = argument;

        let min_value = if d0 > 0.0 && d1 > 0.0 {
            d0 * d1
        } else {
            if d2 < 0.0 && d3 < 0.0 {
                d2 * d3
            } else {
                (d0 * d3).min(d2 * d1)
            }
        };

        let max_value = if d0 > 0.0 && d1 > 0.0 {
            d2 * d3
        } else {
            if d2 < 0.0 && d3 < 0.0 {
                d0 * d1
            } else {
                (d0 * d1).max(d2 * d3)
            }
        };

        Self {
            input,
            argument,
            min_value,
            max_value,
        }
    }
}

impl Debug for MulConst {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "MulConst (argument: {}, min_value: {}, max_value: {})", self.argument, self.min_value, self.max_value)
    }
}

impl DensityFunction for MulConst {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.input.sample(at, ctx) * self.argument
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.input.fill_array(arr, ctx);

        for i in 0..arr.len() {
            arr[i] = arr[i] * self.argument;
        }
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::MulConst(
                Box::new(
                    MulConst::new(
                        self.input.map(mapper),
                        self.argument,
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Mul {
    pub argument1: DensityFunctions,
    pub argument2: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Mul {
    pub fn new(argument1: DensityFunctions, argument2: DensityFunctions) -> Self {
        let d0 = argument1.min_value();
        let d1 = argument2.min_value();
        let d2 = argument1.max_value();
        let d3 = argument2.max_value();

        let min_value = if d0 > 0.0 && d1 > 0.0 {
            d0 * d1
        } else {
            if d2 < 0.0 && d3 < 0.0 {
                d2 * d3
            } else {
                (d0 * d3).min(d2 * d1)
            }
        };

        let max_value = if d0 > 0.0 && d1 > 0.0 {
            d2 * d3
        } else {
            if d2 < 0.0 && d3 < 0.0 {
                d0 * d1
            } else {
                (d0 * d1).max(d2 * d3)
            }
        };

        Self {
            argument1,
            argument2,
            min_value,
            max_value,
        }
    }
}

impl Debug for Mul {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Mul (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Mul {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let d0 = self.argument1.sample(at, ctx);

        // Optimization: do not call the second function if the first is already zero
        if d0 == 0.0 {
            0.0
        } else {
            d0 * self.argument2.sample(at, ctx)
        }
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.argument1.fill_array(arr, ctx);

        for i in 0..arr.len() {
            // Optimization: do not call the second function if the first is already zero
            arr[i] = if arr[i] == 0.0 {
                0.0
            } else {
                ctx.for_index(i as i32);
                arr[i] * self.argument2.sample(ctx.pos(), ctx)
            };
        }
    }

    #[inline]
    fn min_value(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Mul(
                Box::new(
                    Mul::new(
                        self.argument1.map(mapper),
                        self.argument2.map(mapper),
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Min {
    argument1: DensityFunctions,
    argument2: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Min {
    pub fn new(argument1: DensityFunctions, argument2: DensityFunctions) -> Self {
        let d0 = argument1.min_value();
        let d1 = argument2.min_value();
        let d2 = argument1.max_value();
        let d3 = argument2.max_value();

        if d0 >= d3 || d1 >= d2 {
            panic!("Creating a function for non-overlapping inputs: {:?} and {:?}", argument1, argument2)
        }

        let min_value = d0.min(d1);
        let max_value = d2.min(d3);

        Self {
            argument1,
            argument2,
            min_value,
            max_value,
        }
    }
}

impl Debug for Min {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Min (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Min {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let d0 = self.argument1.sample(at, ctx);

        if d0 < self.argument2.min_value() {
            d0
        } else {
            d0.min(self.argument2.sample(at, ctx))
        }
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.argument1.fill_array(arr, ctx);

        let min = self.argument2.min_value();
        for j in 0..arr.len() {
            let d1 = arr[j];
            arr[j] = if d1 < min {
                d1
            } else {
                ctx.for_index(j as i32);
                d1.min(self.argument2.sample(ctx.pos(), ctx))
            };
        }
    }

    #[inline]
    fn min_value(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Min(
                Box::new(
                    Min::new(
                        self.argument1.map(mapper),
                        self.argument2.map(mapper),
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Max {
    argument1: DensityFunctions,
    argument2: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Max {
    pub fn new(argument1: DensityFunctions, argument2: DensityFunctions) -> Self {
        let d0 = argument1.min_value();
        let d1 = argument2.min_value();
        let d2 = argument1.max_value();
        let d3 = argument2.max_value();

        if d0 >= d3 || d1 >= d2 {
            panic!("Creating a function for non-overlapping inputs")
        }

        let min_value = d0.max(d1);
        let max_value = d2.max(d3);

        Self {
            argument1,
            argument2,
            min_value,
            max_value,
        }
    }
}

impl Debug for Max {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Max (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Max {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.argument1.sample(at, ctx).max(self.argument2.sample(at, ctx))
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.argument1.fill_array(arr, ctx);

        let max = self.argument2.max_value();
        for l in 0..arr.len() {
            let d1 = arr[l];
            arr[l] = if d1 > max {
                d1
            } else {
                ctx.for_index(l as i32);
                d1.max(self.argument2.sample(ctx.pos(), ctx))
            };
        }
    }

    #[inline]
    fn min_value(&self) -> f64 {
        self.min_value
    }

    #[inline]
    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Max(
                Box::new(
                    Max::new(
                        self.argument1.map(mapper),
                        self.argument2.map(mapper),
                    )
                )
            )
        )
    }
}
