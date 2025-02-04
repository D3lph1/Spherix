use crate::noise::perlin::octave::MultiOctaveNoiseParameters;
use crate::surface::condition::{AbovePreliminaryCondition, BiomeCondition, Conditions, HoleCondition, NoiseThresholdCondition, NotCondition, SteepMaterialCondition, StoneDepthCondition, TemperatureCondition, VerticalAnchor, VerticalGradientCondition, WaterCondition, YCondition};
use crate::surface::context::Context;
use std::collections::HashSet;
use std::sync::Arc;

pub enum ConditionFactories {
    AbovePreliminary(AbovePreliminaryConditionFactory),
    Biome(BiomeConditionFactory),
    Hole(HoleConditionFactory),
    NoiseThreshold(NoiseThresholdConditionFactory),
    Not(Box<NotConditionFactory>),
    SteepMaterial(SteepMaterialConditionFactory),
    StoneDepth(StoneDepthConditionFactory),
    Temperature(TemperatureConditionFactory),
    VerticalGradient(VerticalGradientConditionFactory),
    Water(WaterConditionFactory),
    YAbove(YAboveConditionFactory)
}

impl<'a> ConditionFactory for ConditionFactories {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        match self {
            ConditionFactories::AbovePreliminary(x) => x.create_condition(ctx),
            ConditionFactories::Biome(x) => x.create_condition(ctx),
            ConditionFactories::Hole(x) => x.create_condition(ctx),
            ConditionFactories::NoiseThreshold(x) => x.create_condition(ctx),
            ConditionFactories::Not(x) => x.create_condition(ctx),
            ConditionFactories::SteepMaterial(x) => x.create_condition(ctx),
            ConditionFactories::StoneDepth(x) => x.create_condition(ctx),
            ConditionFactories::Temperature(x) => x.create_condition(ctx),
            ConditionFactories::VerticalGradient(x) => x.create_condition(ctx),
            ConditionFactories::Water(x) => x.create_condition(ctx),
            ConditionFactories::YAbove(x) => x.create_condition(ctx),
        }
    }
}

pub trait ConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions;
}

pub struct AbovePreliminaryConditionFactory;

impl ConditionFactory for AbovePreliminaryConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::AbovePreliminary(AbovePreliminaryCondition)
    }
}

pub struct BiomeConditionFactory {
    pub biome_is: Arc<HashSet<String>>
}

impl ConditionFactory for BiomeConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::Biome(BiomeCondition {
            biome_is: self.biome_is.clone(),
        })
    }
}

pub struct HoleConditionFactory;

impl ConditionFactory for HoleConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::Hole(HoleCondition)
    }
}

pub struct NoiseThresholdConditionFactory {
    pub noise_name: String,
    pub noise_parameters: MultiOctaveNoiseParameters,
    pub min_threshold: f64,
    pub max_threshold: f64
}

impl ConditionFactory for NoiseThresholdConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        let noise = ctx.entropy_bag.get_or_create_noise(self.noise_name.clone(), &self.noise_parameters);

        Conditions::NoiseThreshold(NoiseThresholdCondition{
            noise,
            min_threshold: self.min_threshold,
            max_threshold: self.max_threshold,
        })
    }
}

pub struct NotConditionFactory(pub ConditionFactories);

impl<'a> ConditionFactory for NotConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::Not(Box::new(NotCondition(self.0.create_condition(ctx))))
    }
}

pub struct SteepMaterialConditionFactory;

impl ConditionFactory for SteepMaterialConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::SteepMaterial(SteepMaterialCondition)
    }
}

pub struct StoneDepthConditionFactory {
    pub surface_type: String,
    pub add_surface_depth: bool,
    pub offset: i32,
    pub secondary_depth_range: i32,
}

impl ConditionFactory for StoneDepthConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::StoneDepth(StoneDepthCondition {
            surface_type: self.surface_type.clone(),
            add_surface_depth: self.add_surface_depth,
            offset: self.offset,
            secondary_depth_range: self.secondary_depth_range,
        })
    }
}

pub struct TemperatureConditionFactory;

impl ConditionFactory for TemperatureConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::Temperature(TemperatureCondition)
    }
}

pub struct VerticalGradientConditionFactory {
    pub random_name: String,
    pub true_at_and_below: VerticalAnchor,
    pub false_at_and_above: VerticalAnchor
}

impl ConditionFactory for VerticalGradientConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        let rng_pos = ctx.entropy_bag.get_or_create_rng_pos(self.random_name.clone());

        Conditions::VerticalGradient(VerticalGradientCondition {
            random: rng_pos,
            true_at_and_below: self.true_at_and_below.resolve_y(&ctx.gen_ctx),
            false_at_and_above: self.false_at_and_above.resolve_y(&ctx.gen_ctx),
        })
    }
}

pub struct WaterConditionFactory {
    pub offset: i32,
    pub surface_depth_multiplier: i32,
    pub add_stone_depth: bool
}

impl ConditionFactory for WaterConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::Water(WaterCondition {
            offset: self.offset,
            surface_depth_multiplier: self.surface_depth_multiplier,
            add_stone_depth: self.add_stone_depth,
        })
    }
}

pub struct YAboveConditionFactory {
    pub anchor: VerticalAnchor,
    pub surface_depth_multiplier: i32,
    pub add_stone_depth: bool
}

impl ConditionFactory for YAboveConditionFactory {
    fn create_condition(&self, ctx: &mut Context) -> Conditions {
        Conditions::Y(YCondition {
            anchor: self.anchor.resolve_y(&ctx.gen_ctx),
            surface_depth_multiplier: self.surface_depth_multiplier,
            add_stone_depth: self.add_stone_depth,
        })
    }
}
