use crate::entities::component_with_inner;
use bevy_ecs::prelude::Component;

pub mod player;
pub mod health;

component_with_inner!(OnGround(bool));
