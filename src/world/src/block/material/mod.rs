use crate::block::material::color::Color;

pub mod color;

#[derive(PartialEq, Eq)]
pub struct Material {
    pub color: Color,
    pub push_reaction: PushReaction,
    pub blocks_motion: bool,
    pub flammable: bool,
    pub liquid: bool,
    pub solid_blocking: bool,
    pub replaceable: bool,
    pub solid: bool
}

impl Material {
    pub const AIR: Material = MaterialBuilder::new(Color::NONE).no_collider().not_solid_blocking().non_solid().replaceable().build();
    pub const STRUCTURAL_AIR: Material = MaterialBuilder::new(Color::NONE).no_collider().not_solid_blocking().non_solid().replaceable().build();
    pub const PORTAL: Material = MaterialBuilder::new(Color::NONE).no_collider().not_solid_blocking().non_solid().not_pushable().build();
    pub const CLOTH_DECORATION: Material = MaterialBuilder::new(Color::WOOL).no_collider().not_solid_blocking().non_solid().flammable().build();
    pub const PLANT: Material = MaterialBuilder::new(Color::PLANT).no_collider().not_solid_blocking().non_solid().destroy_on_push().build();
    pub const WATER_PLANT: Material = MaterialBuilder::new(Color::WATER).no_collider().not_solid_blocking().non_solid().destroy_on_push().build();
    pub const REPLACEABLE_PLANT: Material = MaterialBuilder::new(Color::PLANT).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().flammable().build();
    pub const REPLACEABLE_FIREPROOF_PLANT: Material = MaterialBuilder::new(Color::PLANT).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().build();
    pub const REPLACEABLE_WATER_PLANT: Material = MaterialBuilder::new(Color::WATER).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().build();
    pub const WATER: Material = MaterialBuilder::new(Color::WATER).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().liquid().build();
    pub const BUBBLE_COLUMN: Material = MaterialBuilder::new(Color::WATER).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().liquid().build();
    pub const LAVA: Material = MaterialBuilder::new(Color::FIRE).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().liquid().build();
    pub const TOP_SNOW: Material = MaterialBuilder::new(Color::SNOW).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().build();
    pub const FIRE: Material = MaterialBuilder::new(Color::NONE).no_collider().not_solid_blocking().non_solid().destroy_on_push().replaceable().build();
    pub const DECORATION: Material = MaterialBuilder::new(Color::NONE).no_collider().not_solid_blocking().non_solid().destroy_on_push().build();
    pub const WEB: Material = MaterialBuilder::new(Color::WOOL).no_collider().not_solid_blocking().destroy_on_push().build();
    pub const SCULK: Material = MaterialBuilder::new(Color::COLOR_BLACK).build();
    pub const BUILDABLE_GLASS: Material = MaterialBuilder::new(Color::NONE).build();
    pub const CLAY: Material = MaterialBuilder::new(Color::CLAY).build();
    pub const DIRT: Material = MaterialBuilder::new(Color::DIRT).build();
    pub const GRASS: Material = MaterialBuilder::new(Color::GRASS).build();
    pub const ICE_SOLID: Material = MaterialBuilder::new(Color::ICE).build();
    pub const SAND: Material = MaterialBuilder::new(Color::SAND).build();
    pub const SPONGE: Material = MaterialBuilder::new(Color::COLOR_YELLOW).build();
    pub const SHULKER_SHELL: Material = MaterialBuilder::new(Color::COLOR_PURPLE).build();
    pub const WOOD: Material = MaterialBuilder::new(Color::WOOD).flammable().build();
    pub const NETHER_WOOD: Material = MaterialBuilder::new(Color::WOOD).build();
    pub const BAMBOO_SAPLING: Material = MaterialBuilder::new(Color::WOOD).flammable().destroy_on_push().no_collider().build();
    pub const BAMBOO: Material = MaterialBuilder::new(Color::WOOD).flammable().destroy_on_push().build();
    pub const WOOL: Material = MaterialBuilder::new(Color::WOOL).flammable().build();
    pub const EXPLOSIVE: Material = MaterialBuilder::new(Color::FIRE).flammable().not_solid_blocking().build();
    pub const LEAVES: Material = MaterialBuilder::new(Color::PLANT).flammable().not_solid_blocking().destroy_on_push().build();
    pub const GLASS: Material = MaterialBuilder::new(Color::NONE).not_solid_blocking().build();
    pub const ICE: Material = MaterialBuilder::new(Color::ICE).not_solid_blocking().build();
    pub const CACTUS: Material = MaterialBuilder::new(Color::PLANT).not_solid_blocking().destroy_on_push().build();
    pub const STONE: Material = MaterialBuilder::new(Color::STONE).build();
    pub const METAL: Material = MaterialBuilder::new(Color::METAL).build();
    pub const SNOW: Material = MaterialBuilder::new(Color::SNOW).build();
    pub const HEAVY_METAL: Material = MaterialBuilder::new(Color::METAL).not_pushable().build();
    pub const BARRIER: Material = MaterialBuilder::new(Color::NONE).not_pushable().build();
    pub const PISTON: Material = MaterialBuilder::new(Color::STONE).not_pushable().build();
    pub const MOSS: Material = MaterialBuilder::new(Color::PLANT).destroy_on_push().build();
    pub const VEGETABLE: Material = MaterialBuilder::new(Color::PLANT).destroy_on_push().build();
    pub const EGG: Material = MaterialBuilder::new(Color::PLANT).destroy_on_push().build();
    pub const CAKE: Material = MaterialBuilder::new(Color::NONE).destroy_on_push().build();
    pub const AMETHYST: Material = MaterialBuilder::new(Color::COLOR_PURPLE).build();
    pub const POWDER_SNOW: Material = MaterialBuilder::new(Color::SNOW).non_solid().no_collider().build();
    pub const FROGSPAWN: Material = MaterialBuilder::new(Color::WATER).no_collider().not_solid_blocking().non_solid().destroy_on_push().build();
    pub const FROGLIGHT: Material = MaterialBuilder::new(Color::NONE).build();
    pub const DECORATED_POT: Material = MaterialBuilder::new(Color::TERRACOTTA_RED).destroy_on_push().build();
}

pub struct MaterialBuilder {
    color: Color,
    push_reaction: PushReaction,
    blocks_motion: bool,
    flammable: bool,
    liquid: bool,
    solid_blocking: bool,
    replaceable: bool,
    solid: bool
}

impl MaterialBuilder {
    pub const fn new(color: color::Color) -> Self {
        Self {
            color,
            push_reaction: PushReaction::Normal,
            blocks_motion: true,
            flammable: false,
            liquid: false,
            solid_blocking: true,
            replaceable: false,
            solid: true
        }
    }

    pub const fn liquid(mut self) -> Self {
        self.liquid = true;
        self
    }

    pub const fn non_solid(mut self) -> Self {
        self.solid = false;
        self
    }

    pub const fn no_collider(mut self) -> Self {
        self.blocks_motion = false;
        self
    }

    pub const fn not_solid_blocking(mut self) -> Self {
        self.solid_blocking = false;
        self
    }

    pub const fn flammable(mut self) -> Self {
        self.flammable = true;
        self
    }

    pub const fn replaceable(mut self) -> Self {
        self.replaceable = true;
        self
    }

    pub const fn destroy_on_push(mut self) -> Self {
        self.push_reaction = PushReaction::Destroy;
        self
    }

    pub const fn not_pushable(mut self) -> Self {
        self.push_reaction = PushReaction::Block;
        self
    }

    pub const fn build(self) -> Material {
        Material {
            color: self.color,
            push_reaction: self.push_reaction,
            blocks_motion: self.blocks_motion,
            flammable: self.flammable,
            liquid: self.liquid,
            solid_blocking: self.solid_blocking,
            replaceable: self.replaceable,
            solid: self.solid,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub enum PushReaction {
    Normal,
    Destroy,
    Block,
    Ignore,
    PushOnly
}
