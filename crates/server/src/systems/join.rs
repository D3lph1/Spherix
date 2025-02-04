use std::collections::HashMap;
use std::time::Instant;

use bevy_ecs::prelude::{Commands, Entity, EventWriter, Query, RemovedComponents, Res, With};
use flume::Sender;

use spherix_config::Config;
use spherix_math::vector::Vector3f;
use spherix_proto::io::{VarInt, VarLong};
use spherix_proto::packet::clientbound::{InitializeWorldBorder, PlayMapping, RemoveEntities, ServerData, SetCenterChunk, SynchronizePlayerPosition};
use spherix_proto::packet::clientbound::{PlayerInfoUpdate, PlayerInfoUpdateAction, PlayerInfoUpdateActionAddPlayer, PlayerInfoUpdateActionSet, PlayerInfoUpdateActionUpdateListed};
use spherix_world::chunk::pos::ChunkPos;
use spherix_world::dimension::DimensionKind;

use crate::entities::living::health::Health;
use crate::entities::living::player::xp::Xp;
use crate::entities::living::player::{Client, JoinedAt, KnownChunks, LastKnownPosition, LastKnownRotation, LastSentSetCenterChunkPacket, LoadPropertiesTask, LoadedChunksCounter, Name, Player, PlayerSpawnedEvent, PlayerType, ToSend};
use crate::entities::living::OnGround;
use crate::entities::{Id, Uuid, UuidIdMap};
use crate::perf::GeneralPurposeTaskSender;
use crate::player::{Angle, Position, Rotation};
use crate::server::ClientReceiver;
use crate::systems::{schedule_entity_despawn, spawn_entity};
use crate::world::player::worker::LoadPropertiesTaskResultReceiver;

pub fn on_join(
    clients: Res<ClientReceiver>,
    task_tx: Res<GeneralPurposeTaskSender>,
) {
    for client in clients.0.try_iter() {
        task_tx
            .0
            .send(Box::new(LoadPropertiesTask {
                client,
            }))
            .unwrap();
    }
}

pub fn poll_properties(
    config: Res<Config>,
    task_result_tx: Res<LoadPropertiesTaskResultReceiver>,
    mut commands: Commands,
    mut tx: EventWriter<PlayerSpawnedEvent>,
    all_players: Query<(&Uuid, &Name, &ToSend), With<PlayerType>>
) {
    for res in task_result_tx.0.try_iter() {
        let client = res.client;
        let client_uuid = client.uuid.clone();
        let client_name = client.name.clone();
        let client_to_send = client.to_send.clone();

        let prop = res.properties;

        let pos: Vector3f = prop.pos.into();
        let pos: Position = pos.into();
        // let pos = Vector3f::new(pos.x, pos.y + 1.0, pos.z);
        let rotation = prop.rotation;

        spawn_entity(
            Player {
                marker: PlayerType,
                id: Id::next(),
                uuid: Uuid::new(client.uuid),
                name: Name(client.name.clone()),
                client: Client(client),
                joined_at: JoinedAt(Instant::now()),
                to_send: ToSend(client_to_send.clone()),
                xp: Xp::new(prop.xp_total as u32),
                health: Health::new(prop.health as u32, 20),
                pos: pos.clone(),
                last_known_pos: LastKnownPosition(pos.clone()),
                rotation: Rotation::new(Angle(rotation[0]), Angle(rotation[1])),
                last_known_rotation: LastKnownRotation(Rotation::new(Angle(rotation[0]), Angle(rotation[1]))),
                on_ground: OnGround(true),
                dimension: DimensionKind::from(prop.dimension),
                known_chunks: KnownChunks::default(),
                loaded_chunks_counter: LoadedChunksCounter(0),
                last_sent_set_center_chunk_packet: LastSentSetCenterChunkPacket::default(),
            },
            &mut commands,
            &mut tx,
        );

        client_to_send.send(PlayMapping::ServerData(ServerData {
            motd: "{\"text\":\"Hello world\"}".to_string(),
            icon: None,
            enforces_secure_chat: config.auth.enabled && config.chat.secure,
        })).unwrap();

        client_to_send.send(PlayMapping::InitializeWorldBorder(InitializeWorldBorder {
            x: 0.0,
            z: 0.0,
            old_diameter: 59999968.0,
            new_diameter: 59999968.0,
            speed: VarLong(0),
            portal_teleport_boundary: VarInt(29999984),
            warning_blocks: VarInt(5),
            warning_time: VarInt(15),
        })).unwrap();

        client_to_send.send(PlayMapping::SynchronizePlayerPosition(SynchronizePlayerPosition {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            yaw: rotation[0],
            pitch: rotation[1],
            flags: 0,
            teleport_id: VarInt(1),
        })).unwrap();

        let chunk_pos: ChunkPos = pos.into();

        client_to_send.send(PlayMapping::SetCenterChunk(SetCenterChunk {
            chunk_x: VarInt(chunk_pos.x()),
            chunk_z: VarInt(chunk_pos.z()),
        })).unwrap();

        send_update_player_info_packet(
            (client_uuid.into(), client_name, client_to_send),
            &all_players
        );
    }
}

pub fn send_update_player_info_packet(
    new_player: (Uuid, String, Sender<PlayMapping>),
    all_players: &Query<(&Uuid, &Name, &ToSend), With<PlayerType>>
) {
    let mut packet = PlayerInfoUpdate::try_new(
        PlayerInfoUpdateAction::BITMASK_ADD_PLAYER | PlayerInfoUpdateAction::BITMASK_UPDATE_LISTED
    ).unwrap();

    /// into about player self

    let action_sets = vec![
        PlayerInfoUpdateAction::AddPlayer(PlayerInfoUpdateActionAddPlayer {
            name: new_player.1.clone(),
            properties: vec![],
        }),
        PlayerInfoUpdateAction::UpdateListed(PlayerInfoUpdateActionUpdateListed {
            listed: true,
        }),
    ];

    let mut actions = HashMap::new();
    for action_set in action_sets {
        actions.insert(action_set, ());
    }

    packet.push_action_set(PlayerInfoUpdateActionSet {
        uuid: new_player.0.clone().into(),
        actions,
    }).unwrap();

    ///

    for (uuid, name, _) in all_players.iter() {
        let action_sets = vec![
            PlayerInfoUpdateAction::AddPlayer(PlayerInfoUpdateActionAddPlayer {
                name: name.0.clone(),
                properties: vec![],
            }),
            PlayerInfoUpdateAction::UpdateListed(PlayerInfoUpdateActionUpdateListed {
                listed: true,
            }),
        ];

        let mut actions = HashMap::new();
        for action_set in action_sets {
            actions.insert(action_set, ());
        }

        packet.push_action_set(PlayerInfoUpdateActionSet {
            uuid: uuid.clone().into(),
            actions,
        }).unwrap();
    }

    new_player.2
        .send(PlayMapping::PlayerInfoUpdate(packet))
        .unwrap();

    for (each_client_uuid, name, each_client_to_send) in all_players.iter() {
        if each_client_uuid.eq(&new_player.0) {
            continue;
        }

        let mut packet = PlayerInfoUpdate::try_new(
            PlayerInfoUpdateAction::BITMASK_ADD_PLAYER | PlayerInfoUpdateAction::BITMASK_UPDATE_LISTED
        ).unwrap();

        let action_sets = vec![
            PlayerInfoUpdateAction::AddPlayer(PlayerInfoUpdateActionAddPlayer {
                name: new_player.1.clone(),
                properties: vec![],
            }),
            PlayerInfoUpdateAction::UpdateListed(PlayerInfoUpdateActionUpdateListed {
                listed: true,
            }),
        ];

        let mut actions = HashMap::new();
        for action_set in action_sets {
            actions.insert(action_set, ());
        }

        packet.push_action_set(PlayerInfoUpdateActionSet {
            uuid: new_player.0.clone().into(),
            actions,
        }).unwrap();

        each_client_to_send
            .send(PlayMapping::PlayerInfoUpdate(packet))
            .unwrap();
    }
}

pub fn handle_disconnect(
    uuid_id_map: Res<UuidIdMap>,
    mut commands: Commands,
    all_players: Query<(&Uuid, &ToSend), With<PlayerType>>
) {
    for (uuid, to_send) in all_players.iter() {
        if to_send.is_disconnected() {
            schedule_entity_despawn::<PlayerType>(uuid.clone(), &uuid_id_map, &mut commands);
        }
    }
}

pub fn despawn_player(
    mut removals: RemovedComponents<PlayerType>,
    query: Query<(Entity, &Id, &ToSend)>,
    mut commands: Commands
) {
    for removed_player in removals.read() {
        let (_, removed_id, _) = query.get(removed_player).unwrap();

        for (existing_entity, _, to_send) in query.iter() {
            if existing_entity == removed_player {
                continue;
            }

            to_send
                .send(PlayMapping::RemoveEntities(RemoveEntities {
                    entity_ids: vec![VarInt(removed_id.into())],
                }))
                .unwrap();
        }

        commands.entity(removed_player).despawn();
    }
}
