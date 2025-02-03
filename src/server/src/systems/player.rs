use bevy_ecs::prelude::{Entity, EventReader, EventWriter, Query, Res, Without};
use bevy_ecs::query::With;

use spherix_config::Config;
use spherix_math::vector::{OrderedSquareIter, RadialIter, Vector3f};
use spherix_proto::io::VarInt;
use spherix_proto::packet::clientbound::{PlayMapping as ClientboundPlayMapping, SetHeadRotation, SpawnPlayer};
use spherix_proto::packet::serverbound::PlayMapping;
use spherix_world::chunk::pos::GlobalChunkPos;
use spherix_world::dimension::DimensionKind;

use crate::entities::living::player::{Client, KnownChunks, PlayerNeedChunksEvent, PlayerSpawnedEvent, PlayerType, Spawned, ToSend};
use crate::entities::{Id, Uuid};
use crate::player::{Position, Rotation};
use crate::systems::ok_or_skip;
use crate::systems::packet::{ChatCommandPacketEvent, ChatMessagePacketEvent, KeepAlivePacketEvent, PlayerSessionPacketEvent, SetPlayerPositionAndRotationPacketEvent, SetPlayerPositionPacketEvent, SetPlayerRotationPacketEvent, SwingArmPacketEvent};

pub fn on_spawn(
    config: Res<Config>,
    mut rx: EventReader<PlayerSpawnedEvent>,
    mut tx: EventWriter<PlayerNeedChunksEvent>,
    mut query: Query<(&DimensionKind, &Position, &mut KnownChunks), (With<PlayerType>, Without<Spawned>)>,
) {
    for s in rx.read() {
        let entity: Entity = s.into();
        let res = query.get_mut(entity);

        let (dim, pos, mut known_chunks) = ok_or_skip!(res);

        let chunks_iter =
            RadialIter::new(
                OrderedSquareIter::new(
                    GlobalChunkPos::new(dim.clone(), pos.clone().into()),
                    config.world.view_distance as usize,
                )
            );

        for chunk in chunks_iter.clone() {
            known_chunks.insert(chunk.vec, false);
        }

        tx.send(PlayerNeedChunksEvent {
            entity,
            chunks: Box::new(chunks_iter),
        });
    }
}

pub fn spawn_player_entities(
    new_query: Query<(&Id, &Uuid, &Position, &Rotation, &ToSend), (With<PlayerType>, Without<Spawned>)>,
    query: Query<(&Id, &Uuid, &Position, &Rotation, &ToSend), (With<PlayerType>, With<Spawned>)>,
    mut rx: EventReader<PlayerSpawnedEvent>,
) {
    for new in rx.read() {
        let (
            new_id,
            new_uuid,
            new_pos,
            new_rot,
            new_to_send
        ) = new_query.get(new.0).unwrap();

        for (
            existing_id,
            existing_uuid,
            existing_pos,
            existing_rot,
            existing_to_send
        ) in query.iter() {
            spawn_player_entity(existing_id, existing_uuid, existing_pos, existing_rot, new_to_send);
            spawn_player_entity(new_id, new_uuid, new_pos, new_rot, existing_to_send);
        }
    }
}

pub fn spawn_player_entity(id: &Id, uuid: &Uuid, pos: &Vector3f, rot: &Rotation, to_send: &ToSend) {
    to_send
        .send(ClientboundPlayMapping::SpawnPlayer(SpawnPlayer {
            entity_id: VarInt(id.into()),
            player_uuid: uuid.into(),
            x: pos.x,
            y: pos.y,
            z: pos.z,
            yaw: rot.yaw.into(),
            pitch: rot.pitch.into(),
        }))
        .unwrap();

    to_send
        .send(ClientboundPlayMapping::SetHeadRotation(SetHeadRotation {
            entity_id: VarInt(id.into()),
            head_yaw: rot.yaw.into(),
        }))
        .unwrap();
}

macro_rules! match_packet_emit_event {
    (
        ($packet_variable:ident, $entity_variable:ident)
        $(
            $packet:ident => $event_writer:ident
        ),*
    ) => {
        match $packet_variable {
            $(
                PlayMapping::$packet(packet) => {
                    $event_writer.send(<paste::paste!([< $packet PacketEvent >])>::new(
                            $entity_variable,
                            packet
                        )
                    );
                },
            )*
            _ => {}
        }
    };
}

pub fn poll_packets(
    q: Query<(Entity, &Client), (With<PlayerType>)>,
    q_1: Query<(&Spawned), (With<PlayerType>)>,
    (
        mut keep_alive_tx,
        mut set_player_position_tx,
        mut set_player_rotation_tx,
        mut set_player_position_and_rotation_tx,
        mut player_session_tx,
        mut chat_command_tx,
        mut chat_message_tx,
        mut swing_arm_tx,
    ): (
        EventWriter<KeepAlivePacketEvent>,
        EventWriter<SetPlayerPositionPacketEvent>,
        EventWriter<SetPlayerRotationPacketEvent>,
        EventWriter<SetPlayerPositionAndRotationPacketEvent>,
        EventWriter<PlayerSessionPacketEvent>,
        EventWriter<ChatCommandPacketEvent>,
        EventWriter<ChatMessagePacketEvent>,
        EventWriter<SwingArmPacketEvent>,
    ),
) {
    for (entity, client) in q.iter() {
        for packet in client.0.received.try_iter() {
            match_packet_emit_event!(
                (packet, entity)
                KeepAlive => keep_alive_tx,
                SetPlayerPosition => set_player_position_tx,
                SetPlayerRotation => set_player_rotation_tx,
                SetPlayerPositionAndRotation => set_player_position_and_rotation_tx,
                PlayerSession => player_session_tx,
                ChatCommand => chat_command_tx,
                ChatMessage => chat_message_tx,
                SwingArm => swing_arm_tx
            );
        }
    }
}
