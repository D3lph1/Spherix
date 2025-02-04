use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use spherix_math::vector::Vector3;
use std::fmt::{Debug, Formatter};

#[derive(Clone)]
pub struct Abs {
    argument: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Abs {
    pub fn new(argument: DensityFunctions) -> Self {
        let d0 = argument.min_value();
        let min_value = d0.abs();
        let max_value = argument.max_value().abs();

        Self {
            argument,
            min_value: d0.max(0.0),
            max_value: min_value.max(max_value),
        }
    }
}

impl Debug for Abs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Abs (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Abs {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.argument.sample(at, ctx).abs()
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Abs(
                Box::new(
                    Abs::new(
                        self.argument.map(mapper)
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Square {
    argument: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Square {
    pub fn new(argument: DensityFunctions) -> Self {
        let d0 = argument.min_value();
        let min_value = d0.powf(2.0);
        let max_value = argument.max_value().powf(2.0);

        Self {
            argument,
            min_value: d0.max(0.0),
            max_value: min_value.max(max_value),
        }
    }
}

impl Debug for Square {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Square (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Square {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.argument.sample(at, ctx).powf(2.0)
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Square(
                Box::new(
                    Self::new(self.argument.map(mapper))
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Cube {
    argument: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Cube {
    pub fn new(argument: DensityFunctions) -> Self {
        let min_value = argument.min_value().powf(3.0);
        let max_value = argument.max_value().powf(3.0);

        Self {
            argument,
            min_value,
            max_value,
        }
    }
}

impl Debug for Cube {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Cube (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Cube {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.argument.sample(at, ctx).powf(3.0)
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Cube(
                Box::new(
                    Self::new(self.argument.map(mapper))
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct HalfNegative {
    pub argument: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl HalfNegative {
    pub fn new(argument: DensityFunctions) -> Self {
        let mut min_value = Self::half_negative(argument.min_value());
        let mut max_value = Self::half_negative(argument.max_value());

        Self {
            argument,
            min_value,
            max_value,
        }
    }

    #[inline]
    fn half_negative(val: f64) -> f64 {
        if val > 0.0 { val } else { val * 0.5 }
    }
}

impl Debug for HalfNegative {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "HalfNegative (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for HalfNegative {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        Self::half_negative(self.argument.sample(at, ctx))
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::HalfNegative(
                Box::new(
                    Self::new(
                        self.argument.map(mapper)
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct QuarterNegative {
    argument: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl QuarterNegative {
    #[inline]
    pub fn new(argument: DensityFunctions) -> Self {
        let mut min_value = Self::quarter_negative(argument.min_value());
        let mut max_value = Self::quarter_negative(argument.max_value());

        Self {
            argument,
            min_value,
            max_value,
        }
    }

    #[inline]
    fn quarter_negative(val: f64) -> f64 {
        if val > 0.0 { val } else { val * 0.25 }
    }
}

impl Debug for QuarterNegative {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "QuarterNegative (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for QuarterNegative {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        Self::quarter_negative(self.argument.sample(at, ctx))
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::QuarterNegative(
                Box::new(
                    Self::new(
                        self.argument.map(mapper)
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Squeeze {
    argument: DensityFunctions,
    min_value: f64,
    max_value: f64,
}

impl Squeeze {
    #[inline]
    pub fn new(argument: DensityFunctions) -> Self {
        let mut min_value = Self::squeeze(argument.min_value());
        let mut max_value = Self::squeeze(argument.max_value());

        Self {
            argument,
            min_value,
            max_value,
        }
    }

    #[inline]
    fn squeeze(val: f64) -> f64 {
        let val = val.clamp(-1.0, 1.0);

        val / 2.0 - val * val * val / 24.0
    }
}

impl Debug for Squeeze {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Squeeze (min_value: {}, max_value: {})", self.min_value, self.max_value)
    }
}

impl DensityFunction for Squeeze {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        Self::squeeze(self.argument.sample(at, ctx))
    }

    fn min_value(&self) -> f64 {
        self.min_value
    }

    fn max_value(&self) -> f64 {
        self.max_value
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Squeeze(
                Box::new(
                    Self::new(
                        self.argument.map(mapper)
                    )
                )
            )
        )
    }
}
