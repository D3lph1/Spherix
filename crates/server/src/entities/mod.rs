use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, Ordering};

use bevy_ecs::prelude::{Component, Entity, Resource, World};
use bevy_ecs::system::Command;
use owo_colors::OwoColorize;

pub mod living;

macro_rules! component_with_inner {
    ($name:ident($inner:ty)$(, $extra_derives:ty)*) => {
        #[derive(Component, Clone $(, $extra_derives)*)]
        pub struct $name(pub $inner);

        impl $name {
            #[inline]
            pub fn new(inner: $inner) -> Self {
                Self(inner)
            }

            pub fn inner(&self) -> &$inner {
                &self.0
            }

            pub fn into_inner(self) -> $inner {
                self.0
            }
        }

        impl From<$inner> for $name {
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<&$inner> for $name {
            fn from(value: &$inner) -> Self {
                Self(value.clone())
            }
        }

        impl From<$name> for $inner {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<&$name> for $inner {
            fn from(value: &$name) -> Self {
                value.0.clone()
            }
        }

        impl core::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}

pub(crate) use component_with_inner;

// It seems that with values 0 and 1 players behaves themself strangely (stacks and jerks)
static ID_SEQ: AtomicI32 = AtomicI32::new(2);

component_with_inner!(Id(i32), Debug, Copy);

impl Id {
    pub fn next() -> Self {
        Self(ID_SEQ.fetch_add(1, Ordering::Relaxed))
    }
}

component_with_inner!(Uuid(uuid::Uuid), Debug, PartialEq, Eq, Hash);

pub trait UuidIdentifiable {
    fn uuid(&self) -> Uuid;
}

#[derive(Debug, Resource)]
pub struct UuidIdMap {
    pub forward: HashMap<Uuid, Entity>,
    pub inverse: HashMap<Entity, Uuid>,
}

impl UuidIdMap {
    pub fn new() -> Self {
        Self {
            forward: HashMap::new(),
            inverse: HashMap::new()
        }
    }

    pub fn insert(&mut self, uuid: Uuid, id: Entity) -> Result<(), anyhow::Error> {
        if self.forward.contains_key(&uuid) {
            return Err(
                anyhow::Error::msg(
                    format!("Attempt to insert already existing forward key: {:?}", uuid)
                )
            )
        }

        if self.inverse.contains_key(&id) {
            return Err(
                anyhow::Error::msg(
                    format!("Attempt to insert already existing inverse key: {:?}", id)
                )
            )
        }

        self.forward.insert(uuid.clone(), id);
        self.inverse.insert(id, uuid);

        Ok(())
    }

    pub fn remove_by_uuid(&mut self, uuid: Uuid) ->  Result<(), anyhow::Error> {
        let entity = self.forward.remove(&uuid).unwrap();
        self.inverse.remove(&entity).unwrap();

        Ok(())
    }

    pub fn remove_by_entity(&mut self, entity: Entity) -> Result<(), anyhow::Error> {
        let uuid = self.inverse.remove(&entity).unwrap();
        self.forward.remove(&uuid).unwrap();

        Ok(())
    }
}

pub struct InsertUuidIdToMap {
    pub uuid: Uuid,
    pub id: Entity
}

impl Command for InsertUuidIdToMap {
    fn apply(self, world: &mut World) {
        world.get_resource_mut::<UuidIdMap>()
            .unwrap()
            .insert(self.uuid, self.id)
            .unwrap();
    }
}

pub struct RemoveUuidFromMap {
    pub uuid: Uuid
}

impl Command for RemoveUuidFromMap {
    fn apply(self, world: &mut World) {
        world.get_resource_mut::<UuidIdMap>()
            .unwrap()
            .remove_by_uuid(self.uuid)
            .unwrap();
    }
}
