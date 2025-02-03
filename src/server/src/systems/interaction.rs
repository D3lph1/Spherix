use crate::entities::living::player::{PlayerType, Spawned, ToSend};
use crate::entities::Id;
use crate::systems::ok_or_skip;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Entity, Query, With};
use spherix_proto::io::{Byte, VarInt};
use spherix_proto::packet::clientbound::{EntityAnimation, PlayMapping};

use crate::systems::packet::SwingArmPacketEvent;

pub fn on_swing_hand(
    mut rx: EventReader<SwingArmPacketEvent>,
    query: Query<(Entity, &Id, &ToSend), (With<PlayerType>, With<Spawned>)>
) {
    for event in rx.read() {
        let (_, cause_id, _) = ok_or_skip!(query.get(event.entity));

        let animation = if event.packet.hand == 0 {
            // main hand:
            0
        } else if event.packet.hand == 1 {
            // offhand:
            1
        } else {
            panic!("unexpected hand: {}", event.packet.hand)
        };

        for (entity, id, to_send) in query.iter() {
            if entity == event.entity {
                continue
            }

            to_send
                .send(PlayMapping::EntityAnimation(EntityAnimation {
                    entity_id: VarInt(cause_id.into()),
                    animation: animation as Byte,
                }))
                .unwrap();
        }
    }
}
