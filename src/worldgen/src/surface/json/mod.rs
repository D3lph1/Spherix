use crate::noise::json::deserializer::Deserializer;
use crate::noise::json::Resolver;
use crate::surface::condition_factory::ConditionFactories;
use crate::surface::json::deserializer::condition::{AbovePreliminaryDeserializer, BiomeDeserializer, HoleDeserializer, NoiseThresholdDeserializer, NotDeserializer, SteepMaterialDeserializer, StoneDepthDeserializer, TemperatureDeserializer, VerticalGradientDeserializer, WaterDeserializer, YAboveDeserializer};
use crate::surface::json::deserializer::rule::{BandlandsDeserializer, BlockStateDeserializer, ConditionDeserializer, SequenceDeserializer};
use crate::surface::rule::ClayBands;
use crate::surface::rule_factory::RuleFactories;
use spherix_world::chunk::palette::BlockGlobalPalette;
use std::collections::HashMap;
use std::sync::Arc;

pub mod deserializer;
pub mod resolvable;

pub fn condition_deserializers() -> HashMap<String, Box<dyn Deserializer<ConditionFactories>>> {
    HashMap::from([
        ("minecraft:above_preliminary_surface".to_owned(), cast(AbovePreliminaryDeserializer)),
        ("minecraft:biome".to_owned(), cast(BiomeDeserializer)),
        ("minecraft:hole".to_owned(), cast(HoleDeserializer)),
        ("minecraft:noise_threshold".to_owned(), cast(NoiseThresholdDeserializer)),
        ("minecraft:not".to_owned(), cast(NotDeserializer)),
        ("minecraft:steep".to_owned(), cast(SteepMaterialDeserializer)),
        ("minecraft:stone_depth".to_owned(), cast(StoneDepthDeserializer)),
        ("minecraft:temperature".to_owned(), cast(TemperatureDeserializer)),
        ("minecraft:vertical_gradient".to_owned(), cast(VerticalGradientDeserializer)),
        ("minecraft:water".to_owned(), cast(WaterDeserializer)),
        ("minecraft:y_above".to_owned(), cast(YAboveDeserializer)),
    ])
}

pub fn rule_deserializers(
    condition_resolver: Resolver<ConditionFactories>,
    palette: Arc<BlockGlobalPalette>,
    clay_bands: Arc<ClayBands>
) -> HashMap<String, Box<dyn Deserializer<RuleFactories>>> {
    HashMap::from([
        ("minecraft:sequence".to_owned(), cast(SequenceDeserializer)),
        ("minecraft:condition".to_owned(), cast(ConditionDeserializer {
            condition_resolver
        })),
        ("minecraft:block".to_owned(), cast(BlockStateDeserializer {
            palette
        })),
        ("minecraft:bandlands".to_owned(), cast(BandlandsDeserializer {
            clay_bands,
        }))
    ])
}

#[inline]
fn cast<T, D: Deserializer<T> + 'static>(
    t: D,
) -> Box<dyn Deserializer<T>> {
    Box::new(t)
}
