use std::fmt::{Debug, Formatter};

use bevy_ecs::prelude::Component;

#[derive(Component)]
pub struct Health {
    current: u32,
    total: u32
}

impl Health {
    const MAX_DEFAULT: u32 = 20;

    #[inline]
    pub fn new(current: u32, total: u32) -> Self {
        Self {
            current,
            total
        }
    }

    #[inline]
    pub fn deal_damage(&mut self, damage: u32) {
        self.current = if damage > self.current { 0 } else { self.current - damage }
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.current == 0
    }
}

impl Debug for Health {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Health({}/{})", self.current, self.total)
    }
}


