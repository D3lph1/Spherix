use bevy_ecs::prelude::Entity;
use bevy_ecs::prelude::Event;
use paste::paste;

use spherix_proto::packet::serverbound::{ChatCommand, ChatMessage, KeepAlive, PlayerSession, SetPlayerPosition, SetPlayerPositionAndRotation, SetPlayerRotation, SwingArm};

macro_rules! packet_event {
    ($packet_name:ident) => {
        paste!{
            #[derive(Event)]
            pub struct [<$packet_name PacketEvent>] {
                pub entity: Entity,
                pub packet: $packet_name
            }
        }

        paste!{
            impl [<$packet_name PacketEvent>] {
                pub fn new(entity: Entity, packet: $packet_name) -> Self {
                    Self {
                        entity,
                        packet
                    }
                }
            }
        }
    }
}

packet_event!(KeepAlive);
packet_event!(SetPlayerPosition);
packet_event!(SetPlayerRotation);
packet_event!(SetPlayerPositionAndRotation);
packet_event!(PlayerSession);
packet_event!(ChatCommand);
packet_event!(ChatMessage);
packet_event!(SwingArm);
