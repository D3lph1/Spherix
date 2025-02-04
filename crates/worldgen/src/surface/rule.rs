use crate::noise::perlin::Noise;
use crate::surface::condition::{Condition, Conditions};
use crate::surface::context::Context;
use debug_tree::add_branch_to;
use spherix_math::vector::{Vector3, Vector3f};
use spherix_world::block::state::BlockState;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;

pub trait Mapper {
    fn map(&self, rules: Rules) -> Rules;
}

#[derive(Debug)]
pub enum Rules {
    Sequence(SequenceRule),
    State(StateRule),
    Condition(Box<ConditionRule>),
    Bandlands(BandlandsRule),
    Debug(Box<DebugRule>)
}

impl Rule for Rules {
    fn apply(&self, at: Vector3, ctx: &mut Context) -> Option<Arc<BlockState>> {
        match self {
            Rules::Sequence(x) => x.apply(at, ctx),
            Rules::State(x) => x.apply(at, ctx),
            Rules::Condition(x) => x.apply(at, ctx),
            Rules::Bandlands(x) => x.apply(at, ctx),
            Rules::Debug(x) => x.apply(at, ctx),
        }
    }

    fn map<M: Mapper>(self, mapper: &M) -> Rules {
        match self {
            Rules::Sequence(x) => x.map(mapper),
            Rules::State(x) => x.map(mapper),
            Rules::Condition(x) => x.map(mapper),
            Rules::Bandlands(x) => x.map(mapper),
            Rules::Debug(x) => x.map(mapper),
        }
    }
}

pub trait Rule {
    fn apply(&self, at: Vector3, ctx: &mut Context) -> Option<Arc<BlockState>>;

    fn map<M: Mapper>(self, mapper: &M) -> Rules;
}

pub struct SequenceRule(pub Vec<Rules>);

impl Debug for SequenceRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sequence")
    }
}

impl Rule for SequenceRule {
    fn apply(&self, at: Vector3, ctx: &mut Context) -> Option<Arc<BlockState>> {
        for rule in &self.0 {
            let mb_block = rule.apply(at, ctx);
            if mb_block.is_some() {
                return mb_block;
            }
        }

        None
    }

    fn map<M: Mapper>(self, mapper: &M) -> Rules {
        mapper.map(
            Rules::Sequence(
                SequenceRule(
                    self
                        .0
                        .into_iter()
                        .map(|x| x.map(mapper))
                        .collect()
                )
            )
        )
    }
}

#[derive(Clone)]
pub struct StateRule(pub Arc<BlockState>);

impl Debug for StateRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "State({})", self.0.name())
    }
}

impl Rule for StateRule {
    fn apply(&self, at: Vector3, ctx: &mut Context) -> Option<Arc<BlockState>> {
        Some(self.0.clone())
    }

    fn map<M: Mapper>(self, mapper: &M) -> Rules {
        mapper.map(Rules::State(self))
    }
}

pub struct ConditionRule {
    pub condition: Conditions,
    pub follow_up: Rules
}

impl Debug for ConditionRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Condition({:?})", self.condition)
    }
}

impl Rule for ConditionRule {
    fn apply(&self, at: Vector3, ctx: &mut Context) -> Option<Arc<BlockState>> {
        if self.condition.evaluate(ctx) {
            self.follow_up.apply(at, ctx)
        } else {
            None
        }
    }

    fn map<M: Mapper>(self, mapper: &M) -> Rules {
        mapper.map(Rules::Condition(Box::new(ConditionRule {
            condition: self.condition,
            follow_up: self.follow_up.map(mapper),
        })))
    }
}

pub const CLAY_BANDS_MAX_SIZE: usize = 192;

pub type ClayBands = [Arc<BlockState>; CLAY_BANDS_MAX_SIZE];

pub struct BandlandsRule {
    pub clay_bands: Arc<ClayBands>
}

impl Debug for BandlandsRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bandlands")
    }
}

impl Rule for BandlandsRule {
    fn apply(&self, at: Vector3, ctx: &mut Context) -> Option<Arc<BlockState>> {
        let offset = (4.0 * ctx.entropy_bag.noises.clay_bands_offset.sample(
            Vector3f::new(at.x as f64, 0.0, at.z as f64)
        )).round() as i32;

        Some(
            self.clay_bands[((at.y + offset + self.clay_bands.len() as i32) % self.clay_bands.len() as i32) as usize].clone()
        )
    }

    fn map<M: Mapper>(self, mapper: &M) -> Rules {
        mapper.map(Rules::Bandlands(self))
    }
}

pub trait RuleFactory {
    fn create_rule(&self, ctx: &mut Context) -> Rules;
}

pub struct DebugMapper;

impl Mapper for DebugMapper {
    fn map(&self, rules: Rules) -> Rules {
        Rules::Debug(Box::new(DebugRule(rules)))
    }
}

pub struct DebugRule(Rules);

impl Debug for DebugRule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Debug")
    }
}

impl Rule for DebugRule {
    fn apply(&self, at: Vector3, ctx: &mut Context) -> Option<Arc<BlockState>> {
        let tree = ctx.debug_tree.as_mut().unwrap();
        add_branch_to!(tree.tree, "{:?}: %{}%", self.0, tree.counter);
        let counter = tree.counter;
        tree.counter += 1;

        let sampled = self.0.apply(at, ctx);
        let tree = ctx.debug_tree.as_mut().unwrap();
        match &sampled {
            None => tree.map.insert(counter, "None".to_owned()),
            Some(sampled) => tree.map.insert(counter, sampled.name().to_owned()),
        };

        sampled
    }

    fn map<M: Mapper>(self, mapper: &M) -> Rules {
        self.0.map(mapper)
    }
}
