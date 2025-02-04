use bevy_ecs::prelude::Component;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Component)]
pub enum DimensionKind {
    Overworld,
    TheNether,
    TheEnd,
}

impl From<String> for DimensionKind {
    fn from(value: String) -> Self {
        match value.as_str() {
            "minecraft:overworld" => DimensionKind::Overworld,
            _ => panic!("invalid dimension name")
        }
    }
}
