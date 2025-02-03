use std::time::Duration;

use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::Query;
use bevy_ecs::query::With;

use spherix_proto::packet::clientbound::{Disconnect, PlayMapping};

use crate::entities::living::player::{Client, PlayerType, Spawned};
use crate::entities::Uuid;
use crate::systems::packet::KeepAlivePacketEvent;

pub fn on_keep_alive_packet(
    mut rx: EventReader<KeepAlivePacketEvent>,
    q: Query<(&Client), With<PlayerType>>,
) {
    for event in rx.read() {
        let client = q.get(event.entity).unwrap();
        if !client.0.keep_alive().check(event.packet.keep_alive_id) {
            client.0.send_packet(PlayMapping::Disconnect(Disconnect {
                reason: "{\"text\": \"Invalid keepalive ID\"}".to_string(),
            }));

            return;
        }

        client.0.keep_alive().set_next_id();
    }
}

const KEEP_ALIVE_FREQ: Duration = Duration::from_secs(10);

pub fn keep_alive(
    q: Query<(&Uuid, &Client), (With<PlayerType>, With<Spawned>)>,
) {
    for (uuid, client) in q.iter() {
        if client.0.keep_alive().is_time_to(KEEP_ALIVE_FREQ) {
            client.0.keep_alive().send();
        }
    }
}
