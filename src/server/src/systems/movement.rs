use bevy_ecs::prelude::{Commands, Entity, EventReader, Or, Query};
use bevy_ecs::query::{Changed, With};

use spherix_math::vector::Vector3f;
use spherix_proto::io::VarInt;
use spherix_proto::packet::clientbound::{PlayMapping, RemoveEntities, SetHeadRotation, TeleportEntity, UpdateEntityRotation};

use crate::entities::living::player::{LastKnownPosition, LastKnownRotation, PlayerType, Spawned, ToSend};
use crate::entities::living::OnGround;
use crate::entities::{Id, Uuid};
use crate::player::{Angle, Position, Rotation};
use crate::systems::ok_or_skip;
use crate::systems::packet::{SetPlayerPositionAndRotationPacketEvent, SetPlayerPositionPacketEvent, SetPlayerRotationPacketEvent};
use crate::systems::player::spawn_player_entity;

pub fn on_set_player_position_packet(
    mut rx: EventReader<SetPlayerPositionPacketEvent>,
    query: Query<(&Position, &OnGround), (With<PlayerType>, With<Spawned>)>,
    mut commands: Commands
) {
    for event in rx.read() {
        let (pos, on_ground) = ok_or_skip!(query.get(event.entity));
        if pos.x == event.packet.x && pos.y == event.packet.feet_y && pos.z == event.packet.z {
            return;
        }

        commands.entity(event.entity)
            .insert(Position::from(Vector3f::new(event.packet.x, event.packet.feet_y, event.packet.z)));

        if on_ground.0 == event.packet.on_ground {
            return;
        }

        commands.entity(event.entity)
            .insert(OnGround(event.packet.on_ground));
    }
}

pub fn on_set_player_rotation_packet(
    mut rx: EventReader<SetPlayerRotationPacketEvent>,
    query: Query<(&Rotation, &OnGround), (With<PlayerType>, With<Spawned>)>,
    mut commands: Commands
) {
    for event in rx.read() {
        let (rot, on_ground) = ok_or_skip!(query.get(event.entity));
        if rot.yaw.degrees() == event.packet.yaw && rot.pitch.degrees() == event.packet.pitch {
            return;
        }

        commands.entity(event.entity)
            .insert(Rotation::new(Angle(event.packet.yaw), Angle(event.packet.pitch)));

        if on_ground.0 == event.packet.on_ground {
            return;
        }

        commands.entity(event.entity)
            .insert(OnGround(event.packet.on_ground));
    }
}

pub fn on_set_player_position_and_rotation_packet(
    mut rx: EventReader<SetPlayerPositionAndRotationPacketEvent>,
    query: Query<(&Position, &Rotation, &OnGround), (With<PlayerType>, With<Spawned>)>,
    mut commands: Commands
) {
    for event in rx.read() {
        let (pos, rot, on_ground) = ok_or_skip!(query.get(event.entity));

        if pos.x == event.packet.x && pos.y == event.packet.feet_y && pos.z == event.packet.z
            && rot.yaw.degrees() == event.packet.yaw && rot.pitch.degrees() == event.packet.pitch
        {
            return;
        }

        commands.entity(event.entity)
            .insert(Position::from(Vector3f::new(event.packet.x, event.packet.feet_y, event.packet.z)))
            .insert(Rotation::new(Angle(event.packet.yaw), Angle(event.packet.pitch)));

        if on_ground.0 == event.packet.on_ground {
            return;
        }

        commands.entity(event.entity)
            .insert(OnGround(event.packet.on_ground));
    }
}

/// It appears that Notchain server does not use delta-change packets (such as UpdateEntityPositionAndRotation,
/// UpdateEntityPosition or UpdateEntityRotation). Instead, it always sends more robust TeleportEntity
/// packet.
///
/// This approach significantly increases robustness (in terms of network state synchronization)
/// and decreases code complexity.
pub fn on_position_change(
    changes: Query<(Entity, &Id, &Uuid, &Position, &Rotation, &LastKnownPosition, &LastKnownRotation, &OnGround, &ToSend), (Or<(Changed<Position>, Changed<Rotation>)>, With<Spawned>)>,
    players: Query<(Entity, &Id, &Uuid, &Position, &Rotation, &ToSend), With<Spawned>>,
    mut commands: Commands
) {
    for (entity, id, uuid, pos, rot, last_known_pos, last_known_rot, on_ground, to_send) in changes.iter() {
        let is_pos_changed = pos != &last_known_pos.0;
        let is_rot_changed = rot != &last_known_rot.0;

        if is_pos_changed {
            commands.entity(entity)
                .insert(LastKnownPosition(pos.clone()));
        }

        if is_rot_changed {
            commands.entity(entity)
                .insert(LastKnownRotation(rot.clone()));
        }

        for (another_entity, another_id, another_uuid, another_pos, another_rot, another_to_send) in players.iter() {
            if entity == another_entity {
                continue
            }

            if is_pos_changed {
                // For now, just hardcode spawn/despawn distance
                const SPAWN_DISTANCE: f64 = 30.0;

                let prev_dist = (last_known_pos.0.0.clone() - another_pos.0.clone()).norm();
                let dist = (pos.0.clone() - another_pos.0.clone()).norm();

                if prev_dist <= SPAWN_DISTANCE && dist > SPAWN_DISTANCE {
                    another_to_send
                        .send(PlayMapping::RemoveEntities(RemoveEntities {
                            entity_ids: vec![VarInt(id.into())],
                        }))
                        .unwrap();
                }

                if prev_dist > SPAWN_DISTANCE && dist <= SPAWN_DISTANCE {
                    spawn_player_entity(id, uuid, pos, rot, another_to_send);
                }

                if prev_dist <= SPAWN_DISTANCE && dist > SPAWN_DISTANCE {
                    to_send
                        .send(PlayMapping::RemoveEntities(RemoveEntities {
                            entity_ids: vec![VarInt(another_id.into())],
                        }))
                        .unwrap();
                }

                if prev_dist > SPAWN_DISTANCE && dist <= SPAWN_DISTANCE {
                    spawn_player_entity(another_id, another_uuid, another_pos, another_rot, to_send);
                }

                if dist > SPAWN_DISTANCE {
                    continue
                }
            }

            if is_pos_changed && is_rot_changed {
                another_to_send
                    .send(PlayMapping::TeleportEntity(TeleportEntity {
                        entity_id: VarInt(id.into()),
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        yaw: rot.yaw.into(),
                        pitch: rot.pitch.into(),
                        on_ground: on_ground.0,
                    }))
                    .unwrap();

                another_to_send
                    .send(PlayMapping::SetHeadRotation(SetHeadRotation {
                        entity_id: VarInt(id.into()),
                        head_yaw: rot.yaw.into(),
                    }))
                    .unwrap();

                continue;
            }

            if is_pos_changed {
                another_to_send
                    .send(PlayMapping::TeleportEntity(TeleportEntity {
                        entity_id: VarInt(id.into()),
                        x: pos.x,
                        y: pos.y,
                        z: pos.z,
                        yaw: rot.yaw.into(),
                        pitch: rot.pitch.into(),
                        on_ground: on_ground.0,
                    }))
                    .unwrap();

                continue;
            }

            if is_rot_changed {
                another_to_send
                    .send(PlayMapping::UpdateEntityRotation(UpdateEntityRotation {
                        entity_id: VarInt(id.into()),
                        yaw: rot.yaw.into(),
                        pitch: rot.pitch.into(),
                        on_ground: on_ground.0,
                    }))
                    .unwrap();

                another_to_send
                    .send(PlayMapping::SetHeadRotation(SetHeadRotation {
                        entity_id: VarInt(id.into()),
                        head_yaw: rot.yaw.into(),
                    }))
                    .unwrap();
            }
        }
    }
}

fn handle_pos_change() {
    //
}

// fn calc_delta(last_known_pos: f64, curr_pos: f64) -> Short {
//     ((curr_pos * 32.0 - last_known_pos * 32.0) * 128.0) as Short
// }
