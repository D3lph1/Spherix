use crate::block::material::Material;

pub struct Properties {
    material: Material,
    has_collision: bool,
    // light_emission: (block_state) - f32,
    explosion_resistance: f32,
    destroy_time: f32,
    requires_correct_tool_for_drops: bool,
    is_randomly_ticking: bool,
    friction: f32,
    speed_factor: f32,
    jump_factor: f32,
    can_occlude: bool,
    pub is_air: bool,
    pub is_fluid: bool,
    spawn_particles_on_break: bool
}

impl Properties {
    pub fn material(&self) -> &Material {
        &self.material
    }
}

pub struct PropertiesBuilder {
    material: Material,
    has_collision: bool,
    // light_emission: (block_state) - f32,
    explosion_resistance: f32,
    destroy_time: f32,
    requires_correct_tool_for_drops: bool,
    is_randomly_ticking: bool,
    friction: f32,
    speed_factor: f32,
    jump_factor: f32,
    can_occlude: bool,
    is_air: bool,
    is_fluid: bool,
    spawn_particles_on_break: bool
}

impl PropertiesBuilder {
    pub const fn new(material: Material) -> Self {
        Self {
            material,
            has_collision: true,
            // light_emission: (block_state) - f32,
            explosion_resistance: 0.0,
            destroy_time: 0.0,
            requires_correct_tool_for_drops: false,
            is_randomly_ticking: false,
            friction: 0.6,
            speed_factor: 1.0,
            jump_factor: 1.0,
            can_occlude: true,
            is_air: false,
            is_fluid: false,
            spawn_particles_on_break: true,
        }
    }

    pub const fn no_collision(mut self) -> Self {
        self.has_collision = false;
        self.can_occlude = false;
        self
    }

    pub const fn no_occlusion(mut self) -> Self {
        self.can_occlude = false;
        self
    }

    pub const fn friction(mut self, friction: f32) -> Self {
        self.friction = friction;
        self
    }

    pub const fn speed_factor(mut self, speed_factor: f32) -> Self {
        self.speed_factor = speed_factor;
        self
    }

    pub const fn jump_factor(mut self, jump_factor: f32) -> Self {
        self.jump_factor = jump_factor;
        self
    }

    pub const fn strength(mut self, destroy_time: f32, explosion_resistance: f32) -> Self {
        self.destroy_time(destroy_time).explosion_resistance(explosion_resistance)
    }

    pub const fn instabreak(mut self) -> Self {
        self.strength(0.0, 0.0)
    }

    pub const fn random_ticks(mut self) -> Self {
        self.is_randomly_ticking = true;
        self
    }

    pub const fn air(mut self) -> Self {
        self.is_air = true;
        self
    }

    pub const fn fluid(mut self) -> Self {
        self.is_fluid = true;
        self
    }

    pub const fn requires_correct_tool_for_drops(mut self) -> Self {
        self.requires_correct_tool_for_drops = true;
        self
    }

    pub const fn destroy_time(mut self, destroy_time: f32) -> Self {
        self.destroy_time = destroy_time;
        self
    }

    pub const fn explosion_resistance(mut self, explosion_resistance: f32) -> Self {
        // Function f32::max() is not const. So, write a comparison instead:
        self.explosion_resistance = if explosion_resistance > 0.0 {
            explosion_resistance
        } else {
            0.0
        };

        self
    }

    pub const fn no_particles_on_break(mut self) -> Self {
        self.spawn_particles_on_break = false;
        self
    }

    pub const fn build(self) -> Properties {
        Properties {
            material: self.material,
            has_collision: self.has_collision,
            explosion_resistance: self.explosion_resistance,
            destroy_time: self.destroy_time,
            requires_correct_tool_for_drops: self.requires_correct_tool_for_drops,
            is_randomly_ticking: self.is_randomly_ticking,
            friction: self.friction,
            speed_factor: self.speed_factor,
            jump_factor: self.jump_factor,
            can_occlude: self.can_occlude,
            is_air: self.is_air,
            is_fluid: self.is_fluid,
            spawn_particles_on_break: self.spawn_particles_on_break,
        }
    }
}
