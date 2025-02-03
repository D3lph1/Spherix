use bevy_ecs::event::Event;
use bevy_ecs::prelude::{Bundle, Commands, Component, Entity, EventWriter};
use bevy_ecs::system::Res;

use crate::entities::{InsertUuidIdToMap, RemoveUuidFromMap, Uuid, UuidIdMap, UuidIdentifiable};

pub mod join;
pub mod player;
pub mod command;
pub mod movement;
pub mod packet;
pub mod keep_alive;
pub mod message;
pub mod interaction;

pub fn spawn_entity<B: Bundle + UuidIdentifiable, E: Event + From<Entity>>(
    entity: B,
    commands: &mut Commands,
    event_writer: &mut EventWriter<E>
) -> Entity {
    let uuid = entity.uuid();
    let spawned = commands.spawn(entity);
    let id = spawned.id();

    commands.add(InsertUuidIdToMap {
        uuid,
        id
    });
    event_writer.send(id.into());

    id
}

pub fn schedule_entity_despawn<T: Component>(
    uuid: Uuid,
    uuid_id_map: &Res<UuidIdMap>,
    commands: &mut Commands,
) {
    let entity = uuid_id_map.forward.get(&uuid).unwrap();
    commands.entity(entity.clone()).remove::<T>();

    commands.add(RemoveUuidFromMap {
        uuid
    });
}

// TODO: check for QueryDoesNotMatch only error
macro_rules! ok_or_skip {
    ($exp:expr) => {
        ok_or_skip!($exp, continue)
    };
    ($exp:expr, $exit_expr:expr) => {
        {
            let res = $exp;
            if res.is_err() {
                $exit_expr;
            }

            res.unwrap()
        };
    };
}

pub(crate) use ok_or_skip;
