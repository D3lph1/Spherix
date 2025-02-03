use crate::noise::density::density::{DensityFunction, DensityFunctionContext, DensityFunctions, Mapper};
use crate::noise::density::noise::NoiseHolder;
use crate::noise::math::clamped_map;
use crate::noise::perlin::noise::{LegacyNoise, SupremumNoise};
use crate::noise::perlin::DefaultNoise;
use spherix_math::vector::Vector3;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

#[derive(Clone)]
pub struct Const {
    val: f64,
}

impl Const {
    pub fn new(val: f64) -> Self {
        Self {
            val,
        }
    }
}

impl Debug for Const {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Const ({})", self.val)
    }
}

impl DensityFunction for Const {
    fn sample(&self, _: Vector3, _: &mut DensityFunctionContext) -> f64 {
        self.val
    }

    fn min_value(&self) -> f64 {
        self.val
    }

    fn max_value(&self) -> f64 {
        self.val
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(DensityFunctions::Const(self))
    }
}

#[derive(Clone)]
pub struct YClampedGradient {
    from_y: i32,
    to_y: i32,
    from_value: f64,
    to_value: f64,
}

impl YClampedGradient {
    pub fn new(from_y: i32, to_y: i32, from_value: f64, to_value: f64) -> Self {
        Self {
            from_y,
            to_y,
            from_value,
            to_value,
        }
    }
}

impl Debug for YClampedGradient {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "YClampedGradient (from_y: {}, to_y: {}, from_value: {}, to_value: {})",
            self.from_y,
            self.to_y,
            self.from_value,
            self.to_value
        )
    }
}

impl DensityFunction for YClampedGradient {
    fn sample(&self, at: Vector3, _: &mut DensityFunctionContext) -> f64 {
        clamped_map(at.y as f64, self.from_y as f64, self.to_y as f64, self.from_value, self.to_value)
    }

    fn min_value(&self) -> f64 {
        self.from_value.min(self.to_value)
    }

    fn max_value(&self) -> f64 {
        self.from_value.max(self.to_value)
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::YClampedGradient(self)
        )
    }
}

#[derive(Clone)]
pub struct RangeChoice {
    input: DensityFunctions,
    min_inclusive: f64,
    max_exclusive: f64,
    when_in_range: DensityFunctions,
    when_out_of_range: DensityFunctions,
}

impl RangeChoice {
    pub fn new(
        input: DensityFunctions,
        min_inclusive: f64,
        max_exclusive: f64,
        when_in_range: DensityFunctions,
        when_out_of_range: DensityFunctions,
    ) -> Self {
        Self {
            input,
            min_inclusive,
            max_exclusive,
            when_in_range,
            when_out_of_range,
        }
    }
}

impl Debug for RangeChoice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "RangeChoice (min_inclusive: {}, max_exclusive: {})", self.min_inclusive, self.max_exclusive)
    }
}

impl DensityFunction for RangeChoice {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let d0 = self.input.sample(at, ctx);

        if d0 >= self.min_inclusive && d0 < self.max_exclusive {
            self.when_in_range.sample(at, ctx)
        } else {
            self.when_out_of_range.sample(at, ctx)
        }
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        self.input.fill_array(arr, ctx);

        for i in 0..arr.len() {
            let d0 = arr[i];
            ctx.for_index(i as i32);
            if d0 >= self.min_inclusive && d0 < self.max_exclusive {
                arr[i] = self.when_in_range.sample(ctx.pos(), ctx);
            } else {
                arr[i] = self.when_out_of_range.sample(ctx.pos(), ctx);
            }
        }
    }

    fn min_value(&self) -> f64 {
        self.when_in_range.min_value().min(self.when_out_of_range.min_value())
    }

    fn max_value(&self) -> f64 {
        self.when_in_range.max_value().max(self.when_out_of_range.max_value())
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::RangeChoice(
                Box::new(
                    RangeChoice::new(
                        self.input.map(mapper),
                        self.min_inclusive,
                        self.max_exclusive,
                        self.when_in_range.map(mapper),
                        self.when_out_of_range.map(mapper),
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct Clamp {
    input: DensityFunctions,
    min: f64,
    max: f64,
}

impl Clamp {
    pub fn new(input: DensityFunctions, min: f64, max: f64) -> Self {
        Self {
            input,
            min,
            max,
        }
    }
}

impl Debug for Clamp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Clamp (min: {}, max: {})", self.min, self.max)
    }
}

impl DensityFunction for Clamp {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        self.input.sample(at, ctx).clamp(self.min, self.max)
    }

    fn min_value(&self) -> f64 {
        self.min
    }

    fn max_value(&self) -> f64 {
        self.max
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::Clamp(
                Box::new(
                    Clamp::new(self.input.map(mapper), self.min, self.max)
                )
            )
        )
    }
}

pub struct WeirdScaledSampler {
    pub input: DensityFunctions,
    pub noise: Rc<NoiseHolder<DefaultNoise>>,
    pub rarity_value: RarityValue,
}

impl Clone for WeirdScaledSampler {
    fn clone(&self) -> Self {
        Self {
            input: self.input.clone(),
            noise: Rc::new(self.noise.as_ref().clone()),
            rarity_value: self.rarity_value.clone(),
        }
    }
}

impl WeirdScaledSampler {
    pub fn new(input: DensityFunctions, noise: Rc<NoiseHolder<DefaultNoise>>, rarity_value: RarityValue) -> Self {
        Self {
            input,
            noise,
            rarity_value,
        }
    }
}

impl Debug for WeirdScaledSampler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "WeirdScaledSampler (rarity_value: {})", self.rarity_value)
    }
}

impl DensityFunction for WeirdScaledSampler {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        let p_208441_ = self.input.sample(at, ctx);

        let d0 = self.rarity_value.mapper()(p_208441_);
        let sampled = self.noise.sample(at / d0, 0.0, 0.0).abs();
        d0 * sampled
    }

    fn min_value(&self) -> f64 {
        0.0
    }

    fn max_value(&self) -> f64 {
        self.rarity_value.max_rarity() * self.noise.max_value()
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        mapper.map(
            DensityFunctions::WeirdScaledSampler(
                Box::new(
                    WeirdScaledSampler::new(
                        self.input.map(mapper),
                        self.noise,
                        self.rarity_value,
                    )
                )
            )
        )
    }
}

#[derive(Clone)]
pub enum RarityValue {
    Type1,
    Type2,
}

type RarityMapper = fn(f64) -> f64;

impl Display for RarityValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RarityValue::Type1 => write!(f, "type_1"),
            RarityValue::Type2 => write!(f, "type_2"),
        }
    }
}

impl RarityValue {
    fn mapper(&self) -> RarityMapper {
        match self {
            RarityValue::Type1 => Self::spaghetti_rarity_3d,
            RarityValue::Type2 => Self::spaghetti_rarity_2d
        }
    }

    fn max_rarity(&self) -> f64 {
        match self {
            RarityValue::Type1 => 2.0,
            RarityValue::Type2 => 3.0
        }
    }

    fn spaghetti_rarity_3d(val: f64) -> f64 {
        if val < -0.5 {
            0.75
        } else if val < 0.0 {
            1.0
        } else {
            if val < 0.5 { 1.5 } else { 2.0 }
        }
    }

    fn spaghetti_rarity_2d(val: f64) -> f64 {
        if val < -0.75 {
            0.5
        } else if val < -0.5 {
            0.75
        } else if val < 0.5 {
            1.0
        } else {
            if val < 0.75 { 2.0 } else { 3.0 }
        }
    }
}
