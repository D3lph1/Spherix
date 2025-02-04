use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use crate::surface::condition_factory::ConditionFactories;
use crate::surface::rule::{ClayBands, StateRule};
use crate::surface::rule_factory::{BandlandsRuleFactory, ConditionRuleFactory, RuleFactories, SequenceRuleFactory, StateRuleFactory};
use serde_json::Value;
use spherix_world::block::block::{Block, BLOCKS};
use spherix_world::block::state::BlockState;
use spherix_world::block::variant::{Snowy, Variant};
use spherix_world::chunk::palette::BlockGlobalPalette;
use std::sync::Arc;

pub struct SequenceDeserializer;

impl Deserializer<RuleFactories> for SequenceDeserializer {
    fn deserialize(
        &self,
        json: &Value,
        resolver: &Resolver<RuleFactories>,
    ) -> anyhow::Result<RuleFactories> {
        let Value::Object(map) = json else {
            return Err(anyhow::anyhow!("Expected Object, but given: {:?}", json));
        };

        if !map.contains_key("sequence") {
            return Err(anyhow::anyhow!("No \"sequence\" field present"));
        }

        let sequence = map.get("sequence").unwrap();
        let Value::Array(sequence) = sequence else {
            return Err(anyhow::anyhow!("Expected Array, but given: {:?}", json));
        };

        let mut rules = Vec::with_capacity(sequence.len());
        for value in sequence {
            rules.push(resolver.resolve(value)?);
        }

        Ok(RuleFactories::Sequence(SequenceRuleFactory(rules)))
    }
}

pub struct ConditionDeserializer {
    pub condition_resolver: Resolver<ConditionFactories>,
}

impl Deserializer<RuleFactories> for ConditionDeserializer {
    fn deserialize(
        &self,
        json: &Value,
        resolver: &Resolver<RuleFactories>,
    ) -> anyhow::Result<RuleFactories> {
        let Value::Object(map) = json else {
            return Err(anyhow::anyhow!("Expected Object, but given: {:?}", json));
        };

        if !map.contains_key("if_true") {
            return Err(anyhow::anyhow!("No \"if_true\" field present"));
        }

        let if_true = map.get("if_true").unwrap();

        if !map.contains_key("then_run") {
            return Err(anyhow::anyhow!("No \"then_run\" field present"));
        }

        let then_run = map.get("then_run").unwrap();

        Ok(
            RuleFactories::Condition(
                Box::new(
                    ConditionRuleFactory {
                        condition: self.condition_resolver.resolve(if_true)?,
                        then_run: resolver.resolve(then_run)?
                    }
                )
            )
        )
    }
}

pub struct BlockStateDeserializer {
    pub palette: Arc<BlockGlobalPalette>
}

impl Deserializer<RuleFactories> for BlockStateDeserializer {
    fn deserialize(&self, json: &Value, resolver: &Resolver<RuleFactories>) -> anyhow::Result<RuleFactories> {
        let Value::Object(map) = json else {
            return Err(anyhow::anyhow!("Expected Object, but given: {:?}", json));
        };

        if !map.contains_key("result_state") {
            return Err(anyhow::anyhow!("No \"result_state\" field present"));
        }

        let result_state = map.get("result_state").unwrap();

        let Value::Object(map) = result_state else {
            return Err(anyhow::anyhow!("Expected Object, but given: {:?}", result_state));
        };

        if !map.contains_key("Name") {
            return Err(anyhow::anyhow!("No \"Name\" field present"));
        }

        let name = map.get("Name").unwrap();
        if !name.is_string() {
            return Err(anyhow::anyhow!("Expected String but given: {:?}", name));
        }

        let name = name.as_str().unwrap();

        // TODO: Remove hardcode! Write proper state selection!
        let block_state = if name == "minecraft:grass_block" {
            let objs = self.palette.get_objs_by_index(
                &Block::GRASS_BLOCK
            ).unwrap();

            let block_state = <&Arc<BlockState>>::clone(objs.iter().filter(|x| {
                x.variants_match(vec![Variant::Snowy(Snowy(false))])
            })
                .last()
                .unwrap());

            block_state.clone()
        } else {
            self.palette.get_default_obj_by_index(
                BLOCKS.get(name).unwrap()
            ).unwrap()
        };


        Ok(
            RuleFactories::State(
                StateRuleFactory(StateRule(block_state))
            )
        )
    }
}

pub struct BandlandsDeserializer {
    pub clay_bands: Arc<ClayBands>
}

impl Deserializer<RuleFactories> for BandlandsDeserializer {
    fn deserialize(&self, _: &Value, _: &Resolver<RuleFactories>) -> anyhow::Result<RuleFactories> {
        Ok(RuleFactories::Bandlands(BandlandsRuleFactory {
            clay_bands: self.clay_bands.clone(),
        }))
    }
}
