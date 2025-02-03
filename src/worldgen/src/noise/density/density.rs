use crate::noise::blending::blender::{Blender, BlendingOutput};
use crate::noise::density::cache::{Cache2D, CacheAllInCell, CacheOnce, FlatCache};
use crate::noise::math::{floor_div, floor_mod};
use debug_tree::TreeBuilder;
use spherix_math::vector::Vector3;
use spherix_world::chunk::pos::ChunkPos;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Pointer};
use std::rc::Rc;
use std::sync::Arc;

pub trait Mapper {
    fn map(&self, df: DensityFunctions) -> DensityFunctions;
}

pub struct ChainMapper(Vec<Box<dyn Mapper>>);

impl ChainMapper {
    #[inline]
    pub fn new(mappers: Vec<Box<dyn Mapper>>) -> Self {
        Self(mappers)
    }
}

impl Mapper for ChainMapper {
    fn map(&self, mut df: DensityFunctions) -> DensityFunctions {
        for item in self.0.iter() {
            df = item.map(df)
        }

        df
    }
}

impl From<ChainMapper> for Vec<Box<dyn Mapper>> {
    fn from(value: ChainMapper) -> Self {
        value.0
    }
}

pub struct DoNothingMapper;

impl Mapper for DoNothingMapper {
    fn map(&self, df: DensityFunctions) -> DensityFunctions {
        df
    }
}

pub struct DebugMapper;

impl Mapper for DebugMapper {
    fn map(&self, df: DensityFunctions) -> DensityFunctions {
        DensityFunctions::Debug(Box::new(DebugDensityFunction::new(df)))
    }
}

pub struct SetupNoiseMapper<R: RngPos> {
    rng: Arc<R>,
    noises: RefCell<HashMap<String, Rc<NoiseHolder<DefaultNoise>>>>,
}

impl<R: RngPos> SetupNoiseMapper<R> {
    pub fn new(rng: Arc<R>) -> Self {
        Self {
            rng,
            noises: Default::default(),
        }
    }

    pub fn get_or_create_noise(&self, holder: &NoiseHolder<DefaultNoise>) -> Rc<NoiseHolder<DefaultNoise>> {
        let tag = holder.tag();
        let borrow = self.noises.borrow();
        if borrow.contains_key(tag) {
            borrow.get(tag).unwrap().clone()
        } else {
            drop(borrow);

            let noise = Rc::new(
                holder.with_rng(&mut self.rng.by_hash(tag.to_owned()))
            );
            self.noises.borrow_mut().insert(tag.to_owned(), noise.clone());

            noise
        }
    }
}

impl<R: RngPos> Mapper for SetupNoiseMapper<R> {
    fn map(&self, df: DensityFunctions) -> DensityFunctions {
        match df {
            DensityFunctions::Noise(df) => {
                DensityFunctions::Noise(
                    NoiseDensityFunction::new(
                        self.get_or_create_noise(df.noise()),
                        df.xz_scale(),
                        df.y_scale(),
                    )
                )
            }
            DensityFunctions::OldBlendedNoise(x) => {
                DensityFunctions::OldBlendedNoise(
                    x.with_new_rng(
                        &mut self.rng.by_hash("minecraft:terrain".to_owned())
                    )
                )
            }
            DensityFunctions::ShiftedNoise(df) => {
                let noise = self.get_or_create_noise(df.noise());
                let xz_scale = df.xz_scale();
                let y_scale = df.y_scale();
                let (shift_x, shift_y, shift_z) = df.shifts();

                DensityFunctions::ShiftedNoise(
                    Box::new(
                        ShiftedNoise::new(
                            noise,
                            shift_x,
                            shift_y,
                            shift_z,
                            xz_scale,
                            y_scale,
                        )
                    )
                )
            }
            DensityFunctions::ShiftA(df) => {
                DensityFunctions::ShiftA(ShiftA::new(self.get_or_create_noise(df.noise())))
            }
            DensityFunctions::ShiftB(df) => {
                DensityFunctions::ShiftB(ShiftB::new(self.get_or_create_noise(df.noise())))
            }
            DensityFunctions::WeirdScaledSampler(df) => {
                DensityFunctions::WeirdScaledSampler(
                    Box::new(
                        WeirdScaledSampler::new(
                            df.input,
                            self.get_or_create_noise(&df.noise),
                            df.rarity_value,
                        )
                    )
                )
            }
            _ => df
        }
    }
}

pub struct SetupFlatCacheMapper {
    ctx: RefCell<DensityFunctionContext>,
}

impl SetupFlatCacheMapper {
    pub fn new(ctx: RefCell<DensityFunctionContext>) -> Self {
        Self {
            ctx
        }
    }

    pub fn into_ctx(self) -> DensityFunctionContext {
        self.ctx.into_inner()
    }
}

impl Mapper for SetupFlatCacheMapper {
    fn map(&self, df: DensityFunctions) -> DensityFunctions {
        match df {
            DensityFunctions::FlatCache(df) => {
                DensityFunctions::FlatCache(
                    Box::new(
                        FlatCache::new(
                            df.argument,
                            &mut self.ctx.borrow_mut(),
                            true,
                        )
                    )
                )
            }
            _ => df
        }
    }
}

pub struct SetupInterpolatedMapper {
    cell_count_y: usize,
    cell_count_xz: usize,
}

impl SetupInterpolatedMapper {
    pub fn new(cell_count_y: usize, cell_count_xz: usize) -> Self {
        Self {
            cell_count_y,
            cell_count_xz,
        }
    }
}

impl Mapper for SetupInterpolatedMapper {
    fn map(&self, df: DensityFunctions) -> DensityFunctions {
        match &df {
            DensityFunctions::InterpolatedInner(inner) => {
                DensityFunctions::InterpolatedInner(
                    Rc::new(
                        RefCell::new(
                            InterpolatedInner::new(inner.borrow().argument.clone(), self.cell_count_y, self.cell_count_xz)
                        )
                    )
                )
            }
            _ => df
        }
    }
}

pub struct InterpolatedCollector {
    pub collected: RefCell<Vec<Rc<RefCell<InterpolatedInner>>>>,
}

impl InterpolatedCollector {
    pub fn new() -> Self {
        Self {
            collected: RefCell::new(Vec::new()),
        }
    }
}

impl Mapper for InterpolatedCollector {
    fn map(&self, df: DensityFunctions) -> DensityFunctions {
        match &df {
            DensityFunctions::InterpolatedInner(inner) => {
                self
                    .collected
                    .borrow_mut()
                    .push(inner.clone());

                df
            }
            _ => df
        }
    }
}

pub struct CacheAllInCellCollector(RefCell<Vec<Rc<RefCell<CacheAllInCell>>>>);

impl CacheAllInCellCollector {
    pub fn new() -> Self {
        Self(RefCell::new(Vec::new()))
    }
}

impl Mapper for CacheAllInCellCollector {
    fn map(&self, df: DensityFunctions) -> DensityFunctions {
        match &df {
            DensityFunctions::CacheAllInCell(cache) => {
                self
                    .0
                    .borrow_mut()
                    .push(cache.clone());

                df
            }
            _ => df
        }
    }
}

pub trait DensityFunction: Clone + Debug {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64;

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        ctx.fill_all_directly(arr, self);

        let a = 2;
    }
    
    fn min_value(&self) -> f64;
    
    fn max_value(&self) -> f64;

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions;
}

pub enum DensityFunctions {
    Abs(Box<Abs>),
    Add(Box<Add>),
    AddConst(Box<AddConst>),
    BlendAlpha(BlendAlpha),
    BlendDensity(Box<BlendDensity>),
    BlendOffset(BlendOffset),
    Cache2D(Box<Cache2D>),
    CacheAllInCell(Rc<RefCell<CacheAllInCell>>),
    CacheOnce(Box<CacheOnce>),
    Clamp(Box<Clamp>),
    Const(Const),
    Spline(Box<Spline>),
    Cube(Box<Cube>),
    FlatCache(Box<FlatCache>),
    HalfNegative(Box<HalfNegative>),
    InterpolatedInner(Rc<RefCell<InterpolatedInner>>),
    Interpolated(Box<Interpolated>),
    Marker(Box<Marker>),
    Max(Box<Max>),
    Min(Box<Min>),
    MulConst(Box<MulConst>),
    Mul(Box<Mul>),
    Noise(NoiseDensityFunction),
    OldBlendedNoise(OldBlendedNoise),
    QuarterNegative(Box<QuarterNegative>),
    RangeChoice(Box<RangeChoice>),
    ShiftA(ShiftA),
    ShiftB(ShiftB),
    ShiftedNoise(Box<ShiftedNoise>),
    Square(Box<Square>),
    Squeeze(Box<Squeeze>),
    WeirdScaledSampler(Box<WeirdScaledSampler>),
    YClampedGradient(YClampedGradient),
    Debug(Box<DebugDensityFunction>),
}

impl Clone for DensityFunctions {
    fn clone(&self) -> Self {
        match self {
            DensityFunctions::Abs(x) => DensityFunctions::Abs(x.clone()),
            DensityFunctions::Add(x) => DensityFunctions::Add(x.clone()),
            DensityFunctions::AddConst(x) => DensityFunctions::AddConst(x.clone()),
            DensityFunctions::BlendAlpha(x) => DensityFunctions::BlendAlpha(x.clone()),
            DensityFunctions::BlendDensity(x) => DensityFunctions::BlendDensity(x.clone()),
            DensityFunctions::BlendOffset(x) => DensityFunctions::BlendOffset(x.clone()),
            DensityFunctions::Cache2D(x) => DensityFunctions::Cache2D(x.clone()),
            DensityFunctions::CacheAllInCell(x) => DensityFunctions::CacheAllInCell(x.clone()),
            DensityFunctions::CacheOnce(x) => DensityFunctions::CacheOnce(x.clone()),
            DensityFunctions::Clamp(x) => DensityFunctions::Clamp(x.clone()),
            DensityFunctions::Const(x) => DensityFunctions::Const(x.clone()),
            DensityFunctions::Spline(x) => DensityFunctions::Spline(x.clone()),
            DensityFunctions::Cube(x) => DensityFunctions::Cube(x.clone()),
            DensityFunctions::FlatCache(x) => DensityFunctions::FlatCache(x.clone()),
            DensityFunctions::HalfNegative(x) => DensityFunctions::HalfNegative(x.clone()),
            DensityFunctions::InterpolatedInner(x) => {
                let borrow = x.borrow();

                DensityFunctions::InterpolatedInner(
                    Rc::new(
                        RefCell::new(
                            InterpolatedInner::new(
                                borrow.argument.clone(),
                                borrow.cell_count_y,
                                borrow.cell_count_xz,
                            )
                        )
                    )
                )
            }
            DensityFunctions::Interpolated(x) => DensityFunctions::Interpolated(x.clone()),
            DensityFunctions::Marker(x) => DensityFunctions::Marker(x.clone()),
            DensityFunctions::Max(x) => DensityFunctions::Max(x.clone()),
            DensityFunctions::Min(x) => DensityFunctions::Min(x.clone()),
            DensityFunctions::MulConst(x) => DensityFunctions::MulConst(x.clone()),
            DensityFunctions::Mul(x) => DensityFunctions::Mul(x.clone()),
            DensityFunctions::Noise(x) => DensityFunctions::Noise(x.clone()),
            DensityFunctions::OldBlendedNoise(x) => DensityFunctions::OldBlendedNoise(x.clone()),
            DensityFunctions::QuarterNegative(x) => DensityFunctions::QuarterNegative(x.clone()),
            DensityFunctions::RangeChoice(x) => DensityFunctions::RangeChoice(x.clone()),
            DensityFunctions::ShiftA(x) => DensityFunctions::ShiftA(x.clone()),
            DensityFunctions::ShiftB(x) => DensityFunctions::ShiftB(x.clone()),
            DensityFunctions::ShiftedNoise(x) => DensityFunctions::ShiftedNoise(x.clone()),
            DensityFunctions::Square(x) => DensityFunctions::Square(x.clone()),
            DensityFunctions::Squeeze(x) => DensityFunctions::Squeeze(x.clone()),
            DensityFunctions::WeirdScaledSampler(x) => DensityFunctions::WeirdScaledSampler(x.clone()),
            DensityFunctions::YClampedGradient(x) => DensityFunctions::YClampedGradient(x.clone()),
            DensityFunctions::Debug(x) => DensityFunctions::Debug(x.clone()),
        }
    }
}

impl Debug for DensityFunctions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DensityFunctions::Abs(x) => Debug::fmt(&x, f),
            DensityFunctions::Add(x) => Debug::fmt(&x, f),
            DensityFunctions::AddConst(x) => Debug::fmt(&x, f),
            DensityFunctions::BlendAlpha(x) => Debug::fmt(&x, f),
            DensityFunctions::BlendDensity(x) => Debug::fmt(&x, f),
            DensityFunctions::BlendOffset(x) => Debug::fmt(&x, f),
            DensityFunctions::Cache2D(x) => Debug::fmt(&x, f),
            DensityFunctions::CacheAllInCell(x) => Debug::fmt(&x, f),
            DensityFunctions::CacheOnce(x) => Debug::fmt(&x, f),
            DensityFunctions::Clamp(x) => Debug::fmt(&x, f),
            DensityFunctions::Const(x) => Debug::fmt(&x, f),
            DensityFunctions::Spline(x) => Debug::fmt(&x, f),
            DensityFunctions::Cube(x) => Debug::fmt(&x, f),
            DensityFunctions::FlatCache(x) => Debug::fmt(&x, f),
            DensityFunctions::HalfNegative(x) => Debug::fmt(&x, f),
            DensityFunctions::InterpolatedInner(x) => Debug::fmt(&x.borrow(), f),
            DensityFunctions::Interpolated(x) => Debug::fmt(&x, f),
            DensityFunctions::Marker(x) => Debug::fmt(&x, f),
            DensityFunctions::Max(x) => Debug::fmt(&x, f),
            DensityFunctions::Min(x) => Debug::fmt(&x, f),
            DensityFunctions::MulConst(x) => Debug::fmt(&x, f),
            DensityFunctions::Mul(x) => Debug::fmt(&x, f),
            DensityFunctions::Noise(x) => Debug::fmt(&x, f),
            DensityFunctions::OldBlendedNoise(x) => Debug::fmt(&x, f),
            DensityFunctions::QuarterNegative(x) => Debug::fmt(&x, f),
            DensityFunctions::RangeChoice(x) => Debug::fmt(&x, f),
            DensityFunctions::ShiftA(x) => Debug::fmt(&x, f),
            DensityFunctions::ShiftB(x) => Debug::fmt(&x, f),
            DensityFunctions::ShiftedNoise(x) => Debug::fmt(&x, f),
            DensityFunctions::Square(x) => Debug::fmt(&x, f),
            DensityFunctions::Squeeze(x) => Debug::fmt(&x, f),
            DensityFunctions::WeirdScaledSampler(x) => Debug::fmt(&x, f),
            DensityFunctions::YClampedGradient(x) => Debug::fmt(&x, f),
            DensityFunctions::Debug(x) => Debug::fmt(&x, f),
        }
    }
}

impl DensityFunction for DensityFunctions {
    fn sample(&self, at: Vector3, ctx: &mut DensityFunctionContext) -> f64 {
        match self {
            DensityFunctions::Abs(x) => x.sample(at, ctx),
            DensityFunctions::Add(x) => x.sample(at, ctx),
            DensityFunctions::AddConst(x) => x.sample(at, ctx),
            DensityFunctions::BlendAlpha(x) => x.sample(at, ctx),
            DensityFunctions::BlendDensity(x) => x.sample(at, ctx),
            DensityFunctions::BlendOffset(x) => x.sample(at, ctx),
            DensityFunctions::Cache2D(x) => x.sample(at, ctx),
            DensityFunctions::CacheAllInCell(x) => x.borrow().sample(at, ctx),
            DensityFunctions::CacheOnce(x) => x.sample(at, ctx),
            DensityFunctions::Clamp(x) => x.sample(at, ctx),
            DensityFunctions::Const(x) => x.sample(at, ctx),
            DensityFunctions::Spline(x) => x.sample(at, ctx),
            DensityFunctions::Cube(x) => x.sample(at, ctx),
            DensityFunctions::FlatCache(x) => x.sample(at, ctx),
            DensityFunctions::HalfNegative(x) => x.sample(at, ctx),
            DensityFunctions::InterpolatedInner(x) => x.borrow().sample(at, ctx),
            DensityFunctions::Interpolated(x) => x.sample(at, ctx),
            DensityFunctions::Marker(x) => x.sample(at, ctx),
            DensityFunctions::Max(x) => x.sample(at, ctx),
            DensityFunctions::Min(x) => x.sample(at, ctx),
            DensityFunctions::MulConst(x) => x.sample(at, ctx),
            DensityFunctions::Mul(x) => x.sample(at, ctx),
            DensityFunctions::Noise(x) => x.sample(at, ctx),
            DensityFunctions::OldBlendedNoise(x) => x.sample(at, ctx),
            DensityFunctions::QuarterNegative(x) => x.sample(at, ctx),
            DensityFunctions::RangeChoice(x) => x.sample(at, ctx),
            DensityFunctions::ShiftA(x) => x.sample(at, ctx),
            DensityFunctions::ShiftB(x) => x.sample(at, ctx),
            DensityFunctions::ShiftedNoise(x) => x.sample(at, ctx),
            DensityFunctions::Square(x) => x.sample(at, ctx),
            DensityFunctions::Squeeze(x) => x.sample(at, ctx),
            DensityFunctions::WeirdScaledSampler(x) => x.sample(at, ctx),
            DensityFunctions::YClampedGradient(x) => x.sample(at, ctx),
            DensityFunctions::Debug(x) => x.sample(at, ctx),
        }
    }

    fn fill_array(&self, arr: &mut [f64], ctx: &mut DensityFunctionContext) {
        match self {
            DensityFunctions::Abs(x) => x.fill_array(arr, ctx),
            DensityFunctions::Add(x) => x.fill_array(arr, ctx),
            DensityFunctions::AddConst(x) => x.fill_array(arr, ctx),
            DensityFunctions::BlendAlpha(x) => x.fill_array(arr, ctx),
            DensityFunctions::BlendDensity(x) => x.fill_array(arr, ctx),
            DensityFunctions::BlendOffset(x) => x.fill_array(arr, ctx),
            DensityFunctions::Cache2D(x) => x.fill_array(arr, ctx),
            DensityFunctions::CacheAllInCell(x) => x.borrow().fill_array(arr, ctx),
            DensityFunctions::CacheOnce(x) => x.fill_array(arr, ctx),
            DensityFunctions::Clamp(x) => x.fill_array(arr, ctx),
            DensityFunctions::Const(x) => x.fill_array(arr, ctx),
            DensityFunctions::Spline(x) => x.fill_array(arr, ctx),
            DensityFunctions::Cube(x) => x.fill_array(arr, ctx),
            DensityFunctions::FlatCache(x) => x.fill_array(arr, ctx),
            DensityFunctions::HalfNegative(x) => x.fill_array(arr, ctx),
            DensityFunctions::InterpolatedInner(x) => x.borrow().fill_array(arr, ctx),
            DensityFunctions::Interpolated(x) => x.fill_array(arr, ctx),
            DensityFunctions::Marker(x) => x.fill_array(arr, ctx),
            DensityFunctions::Max(x) => x.fill_array(arr, ctx),
            DensityFunctions::Min(x) => x.fill_array(arr, ctx),
            DensityFunctions::MulConst(x) => x.fill_array(arr, ctx),
            DensityFunctions::Mul(x) => x.fill_array(arr, ctx),
            DensityFunctions::Noise(x) => x.fill_array(arr, ctx),
            DensityFunctions::OldBlendedNoise(x) => x.fill_array(arr, ctx),
            DensityFunctions::QuarterNegative(x) => x.fill_array(arr, ctx),
            DensityFunctions::RangeChoice(x) => x.fill_array(arr, ctx),
            DensityFunctions::ShiftA(x) => x.fill_array(arr, ctx),
            DensityFunctions::ShiftB(x) => x.fill_array(arr, ctx),
            DensityFunctions::ShiftedNoise(x) => x.fill_array(arr, ctx),
            DensityFunctions::Square(x) => x.fill_array(arr, ctx),
            DensityFunctions::Squeeze(x) => x.fill_array(arr, ctx),
            DensityFunctions::WeirdScaledSampler(x) => x.fill_array(arr, ctx),
            DensityFunctions::YClampedGradient(x) => x.fill_array(arr, ctx),
            DensityFunctions::Debug(x) => x.fill_array(arr, ctx),
        }
    }

    fn min_value(&self) -> f64 {
        match self {
            DensityFunctions::Abs(x) => x.min_value(),
            DensityFunctions::Add(x) => x.min_value(),
            DensityFunctions::AddConst(x) => x.min_value(),
            DensityFunctions::BlendAlpha(x) => x.min_value(),
            DensityFunctions::BlendDensity(x) => x.min_value(),
            DensityFunctions::BlendOffset(x) => x.min_value(),
            DensityFunctions::Cache2D(x) => x.min_value(),
            DensityFunctions::CacheAllInCell(x) => x.borrow().min_value(),
            DensityFunctions::CacheOnce(x) => x.min_value(),
            DensityFunctions::Clamp(x) => x.min_value(),
            DensityFunctions::Const(x) => x.min_value(),
            DensityFunctions::Spline(x) => x.min_value(),
            DensityFunctions::Cube(x) => x.min_value(),
            DensityFunctions::FlatCache(x) => x.min_value(),
            DensityFunctions::HalfNegative(x) => x.min_value(),
            DensityFunctions::InterpolatedInner(x) => x.borrow().min_value(),
            DensityFunctions::Interpolated(x) => x.min_value(),
            DensityFunctions::Marker(x) => x.min_value(),
            DensityFunctions::Max(x) => x.min_value(),
            DensityFunctions::Min(x) => x.min_value(),
            DensityFunctions::MulConst(x) => x.min_value(),
            DensityFunctions::Mul(x) => x.min_value(),
            DensityFunctions::Noise(x) => x.min_value(),
            DensityFunctions::OldBlendedNoise(x) => x.min_value(),
            DensityFunctions::QuarterNegative(x) => x.min_value(),
            DensityFunctions::RangeChoice(x) => x.min_value(),
            DensityFunctions::ShiftA(x) => x.min_value(),
            DensityFunctions::ShiftB(x) => x.min_value(),
            DensityFunctions::ShiftedNoise(x) => x.min_value(),
            DensityFunctions::Square(x) => x.min_value(),
            DensityFunctions::Squeeze(x) => x.min_value(),
            DensityFunctions::WeirdScaledSampler(x) => x.min_value(),
            DensityFunctions::YClampedGradient(x) => x.min_value(),
            DensityFunctions::Debug(x) => x.min_value(),
        }
    }

    fn max_value(&self) -> f64 {
        match self {
            DensityFunctions::Abs(x) => x.max_value(),
            DensityFunctions::Add(x) => x.max_value(),
            DensityFunctions::AddConst(x) => x.max_value(),
            DensityFunctions::BlendAlpha(x) => x.max_value(),
            DensityFunctions::BlendDensity(x) => x.max_value(),
            DensityFunctions::BlendOffset(x) => x.max_value(),
            DensityFunctions::Cache2D(x) => x.max_value(),
            DensityFunctions::CacheAllInCell(x) => x.borrow().max_value(),
            DensityFunctions::CacheOnce(x) => x.max_value(),
            DensityFunctions::Clamp(x) => x.max_value(),
            DensityFunctions::Const(x) => x.max_value(),
            DensityFunctions::Spline(x) => x.max_value(),
            DensityFunctions::Cube(x) => x.max_value(),
            DensityFunctions::FlatCache(x) => x.max_value(),
            DensityFunctions::HalfNegative(x) => x.max_value(),
            DensityFunctions::InterpolatedInner(x) => x.borrow().max_value(),
            DensityFunctions::Interpolated(x) => x.max_value(),
            DensityFunctions::Marker(x) => x.max_value(),
            DensityFunctions::Max(x) => x.max_value(),
            DensityFunctions::Min(x) => x.max_value(),
            DensityFunctions::MulConst(x) => x.max_value(),
            DensityFunctions::Mul(x) => x.max_value(),
            DensityFunctions::Noise(x) => x.max_value(),
            DensityFunctions::OldBlendedNoise(x) => x.max_value(),
            DensityFunctions::QuarterNegative(x) => x.max_value(),
            DensityFunctions::RangeChoice(x) => x.max_value(),
            DensityFunctions::ShiftA(x) => x.max_value(),
            DensityFunctions::ShiftB(x) => x.max_value(),
            DensityFunctions::ShiftedNoise(x) => x.max_value(),
            DensityFunctions::Square(x) => x.max_value(),
            DensityFunctions::Squeeze(x) => x.max_value(),
            DensityFunctions::WeirdScaledSampler(x) => x.max_value(),
            DensityFunctions::YClampedGradient(x) => x.max_value(),
            DensityFunctions::Debug(x) => x.max_value(),
        }
    }

    fn map<M: Mapper>(self, mapper: &M) -> DensityFunctions {
        match self {
            DensityFunctions::Abs(x) => x.map(mapper),
            DensityFunctions::Add(x) => x.map(mapper),
            DensityFunctions::AddConst(x) => x.map(mapper),
            DensityFunctions::BlendAlpha(x) => x.map(mapper),
            DensityFunctions::BlendDensity(x) => x.map(mapper),
            DensityFunctions::BlendOffset(x) => x.map(mapper),
            DensityFunctions::Cache2D(x) => x.map(mapper),
            DensityFunctions::CacheAllInCell(x) => {
                let strong_counter = Rc::strong_count(&x);
                let inner = Rc::into_inner(x);
                if inner.is_none() {
                    panic!(
                        concat!(
                        "Rc has {} strong reference(s). To map CacheAllInCell Function ",
                        "it is required for strong counter of Rc to be 1. Do not clone ",
                        "CacheAllInCell Density Function reference without urgent need."
                        ),
                        strong_counter
                    )
                }

                inner.unwrap().into_inner().map(mapper)
            }
            DensityFunctions::CacheOnce(x) => x.map(mapper),
            DensityFunctions::Clamp(x) => x.map(mapper),
            DensityFunctions::Const(x) => x.map(mapper),
            DensityFunctions::Spline(x) => x.map(mapper),
            DensityFunctions::Cube(x) => x.map(mapper),
            DensityFunctions::FlatCache(x) => x.map(mapper),
            DensityFunctions::HalfNegative(x) => x.map(mapper),
            DensityFunctions::InterpolatedInner(x) => {
                let strong_counter = Rc::strong_count(&x);
                let inner = Rc::into_inner(x);
                if inner.is_none() {
                    panic!(
                        concat!(
                        "Rc has {} strong reference(s). To map Interpolated Density Function ",
                        "it is required for strong counter of Rc to be 1. Do not clone ",
                        "Interpolated Density Function reference without urgent need."
                        ),
                        strong_counter
                    )
                }

                inner.unwrap().into_inner().map(mapper)
            }
            DensityFunctions::Interpolated(x) => x.map(mapper),
            DensityFunctions::Marker(x) => x.map(mapper),
            DensityFunctions::Max(x) => x.map(mapper),
            DensityFunctions::Min(x) => x.map(mapper),
            DensityFunctions::MulConst(x) => x.map(mapper),
            DensityFunctions::Mul(x) => x.map(mapper),
            DensityFunctions::Noise(x) => x.map(mapper),
            DensityFunctions::OldBlendedNoise(x) => x.map(mapper),
            DensityFunctions::QuarterNegative(x) => x.map(mapper),
            DensityFunctions::RangeChoice(x) => x.map(mapper),
            DensityFunctions::ShiftA(x) => x.map(mapper),
            DensityFunctions::ShiftB(x) => x.map(mapper),
            DensityFunctions::ShiftedNoise(x) => x.map(mapper),
            DensityFunctions::Square(x) => x.map(mapper),
            DensityFunctions::Squeeze(x) => x.map(mapper),
            DensityFunctions::WeirdScaledSampler(x) => x.map(mapper),
            DensityFunctions::YClampedGradient(x) => x.map(mapper),
            DensityFunctions::Debug(x) => x.map(mapper),
        }
    }
}

/// Used by [`DebugDensityFunction`] to capture tree of Density Function evaluation
pub struct DebugTree<T> {
    pub tree: TreeBuilder,
    pub counter: usize,
    pub map: HashMap<usize, T>,
}

impl<T> DebugTree<T> {
    pub fn new() -> Self {
        Self {
            tree: TreeBuilder::new(),
            counter: 0,
            map: HashMap::new(),
        }
    }

    pub fn reset(&mut self) {
        self.tree = TreeBuilder::new();
        self.counter = 0;
        self.map = HashMap::new();
    }
}

impl<T> Display for DebugTree<T>
where
    T: ToString
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut output = self.tree.string();
        for (k, v) in self.map.iter() {
            output = output.replace(&format!("%{}%", k).to_string(), &v.to_string())
        }

        write!(f, "{}", output)
    }
}

pub struct DensityFunctionContext {
    pub filler: ContextFiller,
    pub filling_cell: bool,
    pub array_index: usize,
    pub in_cell_y: u32,
    pub in_cell_x: u32,
    pub in_cell_z: u32,
    pub cell_width: u32,
    pub cell_height: u32,
    pub cell_count_xz: u32,
    pub cell_count_y: u32,
    pub cell_noise_min_y: i32,
    pub cell_start_block_x: i32,
    pub cell_start_block_y: i32,
    pub cell_start_block_z: i32,
    pub noise_size_xz: u32,
    pub first_cell_x: i32,
    pub first_cell_z: i32,
    pub first_noise_x: i32,
    pub first_noise_z: i32,
    pub interpolating: bool,
    pub interpolation_counter: usize,
    pub array_interpolation_counter: usize,
    pub blender: Blender,
    pub last_blending_data_at: i64,
    pub last_blending_output: BlendingOutput,
    pub debug_tree: Option<DebugTree<f64>>,
}

impl Default for DensityFunctionContext {
    fn default() -> Self {
        Self {
            filler: ContextFiller::Default,
            filling_cell: false,
            array_index: 0,
            in_cell_y: 0,
            in_cell_x: 0,
            in_cell_z: 0,
            cell_width: 0,
            cell_height: 0,
            cell_count_xz: 0,
            cell_count_y: 0,
            cell_noise_min_y: 0,
            cell_start_block_x: 0,
            cell_start_block_y: 0,
            cell_start_block_z: 0,
            noise_size_xz: 0,
            first_cell_x: 0,
            first_cell_z: 0,
            first_noise_x: 0,
            first_noise_z: 0,
            interpolating: false,
            interpolation_counter: 0,
            array_interpolation_counter: 0,
            blender: Blender::new(HashMap::new(), HashMap::new()),
            last_blending_data_at: 0,
            last_blending_output: BlendingOutput { alpha: 0.0, offset: 0.0 },
            debug_tree: None,
        }
    }
}

macro_rules! fill_all_directly {
    ($context_var:ident, $density_func_var:ident, $arr_expr:expr) => {
        match $context_var.filler {
            ContextFiller::Default => {
                $context_var.array_index = 0;

                for i in (0..=$context_var.cell_height - 1).rev() {
                    $context_var.in_cell_y = i;

                    for j in 0..$context_var.cell_width {
                        $context_var.in_cell_x = j;

                        for k in 0..$context_var.cell_width {
                            $context_var.in_cell_z = k;
                            let local_array_index = $context_var.array_index;
                            $context_var.array_index += 1;
                            $arr_expr[local_array_index] = $density_func_var.sample($context_var.pos(), $context_var);
                        }
                    }
                }
            }
            ContextFiller::Slice => {
                for i in 0..=$context_var.cell_count_y as usize {
                    $context_var.cell_start_block_y = (i as i32 + $context_var.cell_noise_min_y) * $context_var.cell_height as i32;
                    $context_var.interpolation_counter += 1;
                    $context_var.in_cell_y = 0;
                    $context_var.array_index = i;
                    $arr_expr[i] = $density_func_var.sample($context_var.pos(), $context_var);
                }
            }
        }
    };
}

impl DensityFunctionContext {
    pub fn for_index(&mut self, index: i32) {
        match self.filler {
            ContextFiller::Default => {
                let i = floor_mod(index, self.cell_width as i32);
                let j = floor_div(index, self.cell_width as i32);
                let k = floor_mod(j, self.cell_width as i32);
                let l = self.cell_height as i32 - 1 - floor_div(j, self.cell_width as i32);
                self.in_cell_x = k as u32;
                self.in_cell_y = l as u32;
                self.in_cell_z = i as u32;
                self.array_index = index as usize;
            }
            ContextFiller::Slice => {
                self.cell_start_block_y = (index + self.cell_noise_min_y) * self.cell_height as i32;
                self.interpolation_counter += 1;
                self.in_cell_y = 0;
                self.array_index = index as usize;
            }
        }
    }

    pub fn get_or_compute_blending_output(&mut self, x: i32, z: i32) -> BlendingOutput {
        let i: i64 = ChunkPos::new(x, z).into();

        if self.last_blending_data_at == i {
            self.last_blending_output.clone()
        } else {
            self.last_blending_data_at = i;
            self.last_blending_output = self.blender.blend_offset_and_factor(x, z);
            self.last_blending_output.clone()
        }
    }

    #[inline]
    pub fn pos(&self) -> Vector3 {
        Vector3::new(
            self.cell_start_block_x + self.in_cell_x as i32,
            self.cell_start_block_y + self.in_cell_y as i32,
            self.cell_start_block_z + self.in_cell_z as i32,
        )
    }

    pub fn fill_all_directly<F: DensityFunction>(&mut self, arr: &mut [f64], f: &F) {
        fill_all_directly!(self, f, arr);
    }
}

#[derive(Clone, PartialEq)]
pub enum ContextFiller {
    Default,
    Slice,
}

pub(crate) use fill_all_directly;
use crate::noise::density::binary::{Add, AddConst, Max, Min, Mul, MulConst};
use crate::noise::density::blend::{BlendAlpha, BlendOffset};
use crate::noise::density::debug::DebugDensityFunction;
use crate::noise::density::maker::{Interpolated, InterpolatedInner, Marker};
use crate::noise::density::misc::{Clamp, Const, RangeChoice, WeirdScaledSampler, YClampedGradient};
use crate::noise::density::noise::{BlendDensity, NoiseDensityFunction, NoiseHolder, OldBlendedNoise, ShiftA, ShiftB, ShiftedNoise};
use crate::noise::density::spline::Spline;
use crate::noise::density::unary::{Abs, Cube, HalfNegative, QuarterNegative, Square, Squeeze};
use crate::noise::perlin::DefaultNoise;
use crate::rng::RngPos;
