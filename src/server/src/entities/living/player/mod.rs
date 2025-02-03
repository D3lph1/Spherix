use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::time::Instant;

use bevy_ecs::prelude::{Bundle, Component, Entity, Event};
use flume::{SendError, Sender};

use spherix_net::client::Client as NetClient;
use spherix_proto::packet::clientbound::{Disconnect, PlayMapping};
use spherix_util::CloneableIterator;
use spherix_world::chunk::pos::{ChunkPos, GlobalChunkPos};
use spherix_world::dimension::DimensionKind;

use crate::entities::living::health::Health;
use crate::entities::living::player::xp::Xp;
use crate::entities::living::OnGround;
use crate::entities::{component_with_inner, Id, Uuid, UuidIdentifiable};
use crate::player::{Position, Rotation};
use crate::world::player::properties::Properties;

pub mod xp;
pub mod food;

#[derive(Event, Debug)]
pub struct PlayerSpawnedEvent(pub Entity);

impl From<Entity> for PlayerSpawnedEvent {
    fn from(value: Entity) -> Self {
        Self(value)
    }
}

impl From<PlayerSpawnedEvent> for Entity {
    fn from(value: PlayerSpawnedEvent) -> Self {
        value.0
    }
}

#[derive(Event)]
pub struct PlayerNeedChunksEvent {
    pub entity: Entity,
    pub chunks: Box<dyn CloneableIterator<Item=GlobalChunkPos> + Send + Sync>
}

#[derive(Event)]
pub struct PlayerUnloadChunksEvent {
    pub entity: Entity,
    pub chunks: Vec<GlobalChunkPos>
}

impl From<&PlayerSpawnedEvent> for Entity {
    fn from(value: &PlayerSpawnedEvent) -> Self {
        value.0
    }
}

#[derive(Event)]
pub struct ChunkDidLoadedEvent {
    pub chunk: GlobalChunkPos
}

#[derive(Event)]
pub struct ChunkDataSentEvent {
    pub entity: Entity,
    pub pos: GlobalChunkPos,
    pub counter: usize
}

pub struct LoadPropertiesTask {
    pub client: NetClient
}

pub struct LoadPropertiesTaskResult {
    pub client: NetClient,
    pub properties: Properties
}

#[derive(Bundle)]
pub struct Player {
    pub marker: PlayerType,
    pub id: Id,
    pub uuid: Uuid,
    pub name: Name,
    pub client: Client,
    pub joined_at: JoinedAt,
    pub to_send: ToSend,
    pub xp: Xp,
    pub health: Health,
    pub pos: Position,
    pub last_known_pos: LastKnownPosition,
    pub rotation: Rotation,
    pub last_known_rotation: LastKnownRotation,
    pub on_ground: OnGround,
    pub dimension: DimensionKind,
    pub known_chunks: KnownChunks,
    pub loaded_chunks_counter: LoadedChunksCounter,
    pub last_sent_set_center_chunk_packet: LastSentSetCenterChunkPacket
}

impl UuidIdentifiable for Player {
    fn uuid(&self) -> Uuid {
        self.uuid.clone()
    }
}

#[derive(Component)]
pub struct PlayerType;

#[derive(Component)]
pub struct Spawned;

component_with_inner!(Name(String));

#[derive(Component)]
pub struct Client(pub NetClient);

component_with_inner!(ToSend(Sender<PlayMapping>));

impl ToSend {
    pub fn disconnect(&self, reason: String) -> Result<(), SendError<PlayMapping>> {
        self.send(PlayMapping::Disconnect(Disconnect {
            reason,
        }))
    }
}

component_with_inner!(JoinedAt(Instant));

component_with_inner!(LastKnownPosition(Position));

component_with_inner!(LastKnownRotation(Rotation));

component_with_inner!(KnownChunks(HashMap<ChunkPos, bool>), Default);

component_with_inner!(LoadedChunksCounter(usize), Default);

pub struct DimensionKindPosPair {
    pub dim: DimensionKind,
    pub pos: Position
}

#[derive(Component, Default)]
pub struct LastSentSetCenterChunkPacket(pub Option<DimensionKindPosPair>);
