use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

use bevy_ecs::change_detection::Res;
use bevy_ecs::event::EventReader;
use bevy_ecs::prelude::{Query, With};
use tracing::{error, info};

use spherix_net::client::Session;
use spherix_proto::io::{Long, VarInt};
use spherix_proto::packet::clientbound::PlayMapping;
use spherix_proto::packet::clientbound::{Disconnect, PlayerChatMessage, PlayerInfoUpdateActionInitializeChat, PlayerInfoUpdateActionInitializeChatSignature};
use spherix_proto::packet::clientbound::{PlayerInfoUpdate, PlayerInfoUpdateAction, PlayerInfoUpdateActionSet};

use crate::entities::living::player::{Client, JoinedAt, PlayerType, Spawned, ToSend};
use crate::entities::{Uuid, UuidIdMap};
use crate::server::Server;
use crate::systems::packet::{ChatMessagePacketEvent, PlayerSessionPacketEvent};

pub fn on_player_session(
    mut rx: EventReader<PlayerSessionPacketEvent>,
    q: Query<(&Uuid, &Client), With<PlayerType>>
) {
    for event in rx.read() {
        let (_, client) = q.get(event.entity).unwrap();

        let session = Session::new(
            event.packet.session_id,
            UNIX_EPOCH + Duration::from_secs(event.packet.expires_at as u64),
            event.packet.public_key.clone(),
            event.packet.key_signature.clone(),
        );

        client.0.set_session(session);

        send_initialize_chat(client, &q);
    }
}

fn send_initialize_chat(send_to: &Client, all_clients: &Query<(&Uuid, &Client), With<PlayerType>>) {
    let mut packet = PlayerInfoUpdate::try_new(
        PlayerInfoUpdateAction::BITMASK_INITIALIZE_CHAT
    ).unwrap();

    for (uuid, each_client) in all_clients {
        if uuid.eq(&send_to.0.uuid.into()) {
            continue;
        }

        let session_lock = each_client.0.session.lock().unwrap();
        let session = session_lock.as_ref().unwrap();
        let action_sets = vec![
            PlayerInfoUpdateAction::InitializeChat(PlayerInfoUpdateActionInitializeChat {
                signature: Some(
                    PlayerInfoUpdateActionInitializeChatSignature {
                        chat_session_id: session.session_id,
                        public_key_expiry_time: session
                            .expires_at
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_millis() as Long,
                        encoded_public_key: session.public_key.clone(),
                        public_key_signature: session.key_signature.clone(),
                    }
                ),
            }),
        ];
        drop(session_lock);

        let mut actions = HashMap::new();
        for action_set in action_sets {
            actions.insert(action_set, ());
        }

        packet.push_action_set(PlayerInfoUpdateActionSet {
            uuid: uuid.clone().into(),
            actions,
        }).unwrap();
    }

    send_to.0
        .to_send
        .send(PlayMapping::PlayerInfoUpdate(packet.clone()))
        .unwrap();

    //

    let mut packet = PlayerInfoUpdate::try_new(
        PlayerInfoUpdateAction::BITMASK_INITIALIZE_CHAT
    ).unwrap();

    let session_lock = send_to.0.session.lock().unwrap();
    let session = session_lock.as_ref().unwrap();
    let action_sets = vec![
        PlayerInfoUpdateAction::InitializeChat(PlayerInfoUpdateActionInitializeChat {
            signature: Some(
                PlayerInfoUpdateActionInitializeChatSignature {
                    chat_session_id: session.session_id,
                    public_key_expiry_time: session
                        .expires_at
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as Long,
                    encoded_public_key: session.public_key.clone(),
                    public_key_signature: session.key_signature.clone(),
                }
            ),
        }),
    ];
    drop(session_lock);

    let mut actions = HashMap::new();
    for action_set in action_sets {
        actions.insert(action_set, ());
    }

    packet.push_action_set(PlayerInfoUpdateActionSet {
        uuid: send_to.0.uuid.clone(),
        actions,
    }).unwrap();

    for (_, each_client) in all_clients {
        each_client.0
            .to_send
            .send(PlayMapping::PlayerInfoUpdate(packet.clone()))
            .unwrap();
    }
}

pub fn on_chat_message_packet(
    mut rx: EventReader<ChatMessagePacketEvent>,
    query: Query<(&Uuid, &Client, &ToSend), (With<PlayerType>, With<Spawned>)>,
) {
    for event in rx.read() {
        let (_, sender, _) = query.get(event.entity).unwrap();

        let message = sender.0.handle_chat(event.packet.clone());
        if message.is_none() {
            error!("Failed to validate message acknowledgements from {}", sender.0.name);

            sender.0
                .to_send
                .send(PlayMapping::Disconnect(Disconnect {
                    reason: "{\"text\": \"Chat message validation failure\"}".to_string(),
                }))
                .unwrap();
        }

        let message = message.unwrap();

        for (uuid, receiver, to_send) in query.iter() {
            to_send.send(PlayMapping::PlayerChatMessage(PlayerChatMessage {
                sender: sender.0.uuid.clone().into(),
                index: VarInt(sender.0.message_index() as i32), // The Notchain server always send 0 when secure chat is disabled
                message_signature: event.packet.signature.clone(),
                body: message.body.pack(&receiver.0.message_signature_cache.lock().unwrap()),
                unsigned_content: None,
                filter_type: VarInt(0),
                filter_type_bits: None,
                chat_type: VarInt(0), // see codec value for /tell <user> <msg> command
                network_name: network_name(&sender.0.uuid.clone().into(), &sender.0.name),
                network_target_name: None,
            })).unwrap();

            receiver.0.add_pending_message(message.clone());
        }

        log_message(&sender.0.name, &message.body.content, message.is_chat_trusted());

        sender.0.inc_message_index();
    }
}

fn log_message(sender_name: &str, content: &str, is_trusted: bool) {
    if is_trusted {
        info!("<{}> {}", sender_name, content);
    } else {
        info!("[Not Secure] <{}> {}", sender_name, content);
    }
}

fn network_name(uuid: &Uuid, username: &String) -> String {
    format!("{{\"insertion\":\"{username}\",\"clickEvent\":{{\"action\":\"suggest_command\",\"value\":\"/tell {username} \"}},\"hoverEvent\":{{\"action\":\"show_entity\",\"contents\":{{\"type\":\"minecraft:player\",\"id\":\"{}\",\"name\":{{\"text\":\"{username}\"}}}}}},\"text\":\"{username}\"}}", uuid.0).to_string()
}
