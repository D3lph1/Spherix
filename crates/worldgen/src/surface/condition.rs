use crate::biome::temperature::cold_enough_to_snow;
use crate::noise::math::map;
use crate::noise::perlin::noise::Noise;
use crate::noise::perlin::DefaultNoise;
use crate::rng::{Rng, RngPos, XoroShiroPos};
use crate::surface::context::{Context, WorldGenerationContext};
use spherix_math::vector::{Vector3, Vector3f};
use spherix_world::chunk::vector::block::Vector2BlockSection;
use std::cmp::{max, min};
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub trait Condition {
    fn evaluate(&self, ctx: &mut Context) -> bool;
}

#[derive(Debug)]
pub enum Conditions {
    AbovePreliminary(AbovePreliminaryCondition),
    Biome(BiomeCondition),
    Hole(HoleCondition),
    NoiseThreshold(NoiseThresholdCondition),
    Not(Box<NotCondition>),
    SteepMaterial(SteepMaterialCondition),
    StoneDepth(StoneDepthCondition),
    Temperature(TemperatureCondition),
    VerticalGradient(VerticalGradientCondition),
    Water(WaterCondition),
    Y(YCondition)
}

impl<'a> Condition for Conditions {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        match self {
            Conditions::AbovePreliminary(x) => x.evaluate(ctx),
            Conditions::Biome(x) => x.evaluate(ctx),
            Conditions::Hole(x) => x.evaluate(ctx),
            Conditions::NoiseThreshold(x) => x.evaluate(ctx),
            Conditions::Not(x) => x.evaluate(ctx),
            Conditions::SteepMaterial(x) => x.evaluate(ctx),
            Conditions::StoneDepth(x) => x.evaluate(ctx),
            Conditions::Temperature(x) => x.evaluate(ctx),
            Conditions::VerticalGradient(x) => x.evaluate(ctx),
            Conditions::Water(x) => x.evaluate(ctx),
            Conditions::Y(x) => x.evaluate(ctx),
        }
    }
}

#[derive(Debug)]
pub struct AbovePreliminaryCondition;

impl Condition for AbovePreliminaryCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        // if ctx.surface_level.block_x == 112 && ctx.surface_level.block_z == 176 {
        //     println!("@");
        // }

        ctx.surface_level.block.y() >= ctx.surface_level.min_surface_level()
    }
}

pub struct BiomeCondition {
    pub biome_is: Arc<HashSet<String>>
}

impl Debug for BiomeCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "BiomeCondition(biome_is: [{}])",
            self
                .biome_is
                .iter()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

impl Condition for BiomeCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        self.biome_is.contains(ctx.biome_gradient.unwrap().biome().name_ref())
    }
}

#[derive(Debug)]
pub struct HoleCondition;

impl Condition for HoleCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        ctx.surface_level.surface_depth <= 0
    }
}

pub struct NoiseThresholdCondition {
    pub noise: Arc<DefaultNoise>,
    pub min_threshold: f64,
    pub max_threshold: f64
}

impl Debug for NoiseThresholdCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NoiseThresholdCondition(min_threshold: {}, max_threshold: {})", self.min_threshold, self.max_threshold)
    }
}

impl Condition for NoiseThresholdCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        let d0 = self.noise.sample(
            Vector3f::new(ctx.surface_level.block.x() as f64, 0.0, ctx.surface_level.block.z() as f64)
        );
        d0 >= self.min_threshold && d0 <= self.max_threshold
    }
}

pub struct NotCondition(pub Conditions);

impl Debug for NotCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "NotCondition({:?})", self.0)
    }
}

impl Condition for NotCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        !self.0.evaluate(ctx)
    }
}

#[derive(Debug)]
pub struct SteepMaterialCondition;

impl Condition for SteepMaterialCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        let i = (ctx.surface_level.block.x() & 15) as u32;
        let j = (ctx.surface_level.block.z() & 15) as u32;
        let k = max(j - 1, 0);
        let l = min(j + 1, 15);

        let i1 = ctx.heightmaps.world_surface_wg
            .as_ref()
            .unwrap()
            .height_section(Vector2BlockSection::new(i, k));

        let j1 = ctx.heightmaps.world_surface_wg
            .as_ref()
            .unwrap()
            .height_section(Vector2BlockSection::new(i, l));

        if j1 >= i1 + 4 {
            true
        } else {
            let k1 = max(i - 1, 0);
            let l1 = min(i + 1, 15);
            let i2 = ctx.heightmaps.world_surface_wg
                .as_ref()
                .unwrap()
                .height_section(Vector2BlockSection::new(k1, j));

            let j2 = ctx.heightmaps.world_surface_wg
                .as_ref()
                .unwrap()
                .height_section(Vector2BlockSection::new(l1, j));

            i2 >= j2 + 4
        }
    }
}

pub struct StoneDepthCondition {
    pub surface_type: String,
    pub add_surface_depth: bool,
    pub offset: i32,
    pub secondary_depth_range: i32,
}

impl Debug for StoneDepthCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "StoneDepthCondition(surface_type: {}, add_surface_depth: {}, offset: {}, secondary_depth_range: {})",
            self.surface_type,
            self.add_surface_depth,
            self.offset,
            self.secondary_depth_range
        )
    }
}

impl Condition for StoneDepthCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        // if ctx.surface_level.block_x == 112 && ctx.surface_level.block_z == 176 {
        //     println!("!")
        // }

        let flag = self.surface_type == "ceiling";
        let i = if flag { ctx.surface_level.stone_depth_below } else { ctx.surface_level.stone_depth_above };
        let j = if self.add_surface_depth { ctx.surface_level.surface_depth } else { 0 };
        let k = if self.secondary_depth_range == 0 {
            0
        } else {
            map(ctx.surface_level.surface_secondary(), -1.0, 1.0, 0.0, self.secondary_depth_range as f64) as i32
        };
        i <= 1 + self.offset + j + k
    }
}

#[derive(Debug)]
pub struct TemperatureCondition;

impl Condition for TemperatureCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        cold_enough_to_snow(
            &ctx.surface_level.block,
            &ctx.biome_gradient.unwrap().biome()
        )
    }
}

pub enum VerticalAnchor {
    Absolute {
        y: i32
    },
    AboveBottom {
        offset: i32
    },
    BelowTop {
        offset: i32
    }
}

impl VerticalAnchor {
    pub fn resolve_y(&self, gen_ctx: &WorldGenerationContext) -> i32 {
        match self {
            VerticalAnchor::Absolute { y } => *y,
            VerticalAnchor::AboveBottom { offset } => gen_ctx.min_y + offset,
            VerticalAnchor::BelowTop { offset } => gen_ctx.height - 1 + gen_ctx.min_y - offset
        }
    }
}

pub struct VerticalGradientCondition {
    pub random: Arc<XoroShiroPos>,
    pub true_at_and_below: i32,
    pub false_at_and_above: i32
}

impl Debug for VerticalGradientCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "VerticalGradientCondition(true_at_and_below: {}, false_at_and_above: {})",
            self.true_at_and_below,
            self.false_at_and_above
        )
    }
}

impl<'a> Condition for VerticalGradientCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        let k = ctx.surface_level.block.y();
        if k <= self.true_at_and_below {
            true
        } else if k >= self.false_at_and_above {
            false
        } else {
            let d0 = map(k as f64, self.true_at_and_below as f64, self.false_at_and_above as f64, 1.0, 0.0);
            let mut rng = self.random.at(Vector3::new(ctx.surface_level.block.x(), k, ctx.surface_level.block.z()));

            (rng.next_f32() as f64) < d0
        }
    }
}

pub struct WaterCondition {
    pub offset: i32,
    pub surface_depth_multiplier: i32,
    pub add_stone_depth: bool
}

impl Debug for WaterCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WaterCondition(offset: {}, surface_depth_multiplier: {}, add_stone_depth: {})",
            self.offset,
            self.surface_depth_multiplier,
            self.add_stone_depth
        )
    }
}

impl Condition for WaterCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        ctx.surface_level.water_height == i32::MIN || ctx.surface_level.block.y() + (if self.add_stone_depth { ctx.surface_level.stone_depth_above } else { 0 })
            >= ctx.surface_level.water_height +self.offset + ctx.surface_level.surface_depth * self.surface_depth_multiplier
    }
}

pub struct YCondition {
    pub anchor: i32,
    pub surface_depth_multiplier: i32,
    pub add_stone_depth: bool
}

impl Debug for YCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "YCondition(anchor: {}, surface_depth_multiplier: {}, add_stone_depth: {})",
            self.anchor,
            self.surface_depth_multiplier,
            self.add_stone_depth
        )
    }
}

impl Condition for YCondition {
    fn evaluate(&self, ctx: &mut Context) -> bool {
        ctx.surface_level.block.y() + (if self.add_stone_depth { ctx.surface_level.stone_depth_above } else { 0 }) >= self.anchor + ctx.surface_level.surface_depth * self.surface_depth_multiplier
    }
}
