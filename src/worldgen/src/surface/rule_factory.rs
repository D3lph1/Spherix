use crate::surface::condition_factory::{ConditionFactories, ConditionFactory};
use crate::surface::context::Context;
use crate::surface::rule::{BandlandsRule, ClayBands, ConditionRule, Rules, SequenceRule, StateRule};
use std::sync::Arc;

pub enum RuleFactories {
    Sequence(SequenceRuleFactory),
    State(StateRuleFactory),
    Condition(Box<ConditionRuleFactory>),
    Bandlands(BandlandsRuleFactory),
}

impl RuleFactory for RuleFactories {
    fn create_rule(&self, ctx: &mut Context) -> Rules {
        match self {
            RuleFactories::Sequence(x) => x.create_rule(ctx),
            RuleFactories::State(x) => x.create_rule(ctx),
            RuleFactories::Condition(x) => x.create_rule(ctx),
            RuleFactories::Bandlands(x) => x.create_rule(ctx),
        }
    }
}

pub trait RuleFactory {
    fn create_rule(&self, ctx: &mut Context) -> Rules;
}

pub struct SequenceRuleFactory(pub Vec<RuleFactories>);

impl RuleFactory for SequenceRuleFactory {
    fn create_rule(&self, ctx: &mut Context) -> Rules {
        Rules::Sequence(
            SequenceRule(
                self
                    .0
                    .iter()
                    .map(|f| f.create_rule(ctx))
                    .collect()
            )
        )
    }
}

pub struct StateRuleFactory(pub StateRule);

impl RuleFactory for StateRuleFactory {
    fn create_rule(&self, _: &mut Context) -> Rules {
        Rules::State(self.0.clone())
    }
}

pub struct ConditionRuleFactory {
    pub condition: ConditionFactories,
    pub then_run: RuleFactories
}

impl RuleFactory for ConditionRuleFactory {
    fn create_rule(&self, ctx: &mut Context) -> Rules {
        Rules::Condition(Box::new(ConditionRule {
            condition: self.condition.create_condition(ctx),
            follow_up: self.then_run.create_rule(ctx),
        }))
    }
}

pub struct BandlandsRuleFactory {
    pub clay_bands: Arc<ClayBands>
}

impl RuleFactory for BandlandsRuleFactory {
    fn create_rule(&self, _: &mut Context) -> Rules {
        Rules::Bandlands(BandlandsRule {
            clay_bands: self.clay_bands.clone(),
        })
    }
}
