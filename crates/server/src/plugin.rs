use bevy_app::{App, Plugin, PostUpdate, Update};
use bevy_ecs::prelude::{IntoSystemConfigs, IntoSystemSetConfigs, SystemSet};
use bevy_ecs::schedule::{LogLevel, ScheduleBuildSettings};

use crate::entities::living::player::{ChunkDataSentEvent, ChunkDidLoadedEvent, PlayerNeedChunksEvent, PlayerSpawnedEvent, PlayerUnloadChunksEvent};
use crate::entities::UuidIdMap;
use crate::systems::command::{on_chat_command_packet, on_command, poll_commands, ChatCommandEvent};
use crate::systems::interaction::on_swing_hand;
use crate::systems::join::{despawn_player, handle_disconnect, on_join, poll_properties};
use crate::systems::keep_alive::{keep_alive, on_keep_alive_packet};
use crate::systems::message::{on_chat_message_packet, on_player_session};
use crate::systems::movement::{on_position_change, on_set_player_position_and_rotation_packet, on_set_player_position_packet, on_set_player_rotation_packet};
use crate::systems::packet::{ChatCommandPacketEvent, ChatMessagePacketEvent, KeepAlivePacketEvent, PlayerSessionPacketEvent, SetPlayerPositionAndRotationPacketEvent, SetPlayerPositionPacketEvent, SetPlayerRotationPacketEvent, SwingArmPacketEvent};
use crate::systems::player::{on_spawn, poll_packets, spawn_player_entities};
use crate::world::dimension::{last_sent_set_center_chunk, load_chunks, on_chunk_data_sent, on_load_event, on_player_movement, poll_chunks, poll_unload_chunks_events};

pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        // resources
        app.insert_resource(UuidIdMap::new());

        // systems
        app
            .add_systems(
                Update,
                (
                    (
                        spawn_player_entities.after(poll_properties),
                        on_player_movement,
                        last_sent_set_center_chunk,
                        poll_unload_chunks_events.after(on_player_movement),
                        load_chunks.after(on_player_movement),
                        poll_chunks,
                        on_load_event
                            .after(load_chunks)
                            .after(poll_chunks),
                        on_chunk_data_sent.after(on_load_event),
                        handle_disconnect,
                        poll_packets,
                    ),
                    (
                        keep_alive,
                        poll_commands.after(on_command),
                        on_command.after(PacketHandler), // ?
                        poll_properties,
                        on_join,
                        on_spawn
                            .before(on_join) // is it needed?
                            .after(poll_properties)
                            .before(load_chunks),
                    ),
                    (
                        on_set_player_position_packet,
                        on_set_player_rotation_packet,
                        on_set_player_position_and_rotation_packet,
                        on_keep_alive_packet,
                        on_chat_command_packet,
                        on_chat_message_packet,
                        on_player_session,
                        on_swing_hand,
                    ).in_set(PacketHandler),
                    (
                        on_position_change.after(PacketHandler),
                    )
                ),
            )
            .add_systems(
                PostUpdate,
                (
                    despawn_player,
                ),
            );

        app
            .configure_sets(
                Update,
                (
                    PacketHandler.after(poll_packets),
                )
            );

        app.edit_schedule(Update, |schedule| {
            schedule.set_build_settings(ScheduleBuildSettings {
                ambiguity_detection: LogLevel::Warn,
                ..Default::default()
            });
        });

        // events
        app
            .add_event::<PlayerSpawnedEvent>()
            .add_event::<PlayerNeedChunksEvent>()
            .add_event::<PlayerUnloadChunksEvent>()
            .add_event::<ChunkDataSentEvent>()
            .add_event::<ChunkDidLoadedEvent>()
            .add_event::<ChatCommandEvent>()
            .add_event::<KeepAlivePacketEvent>()
            .add_event::<SetPlayerPositionPacketEvent>()
            .add_event::<SetPlayerRotationPacketEvent>()
            .add_event::<SetPlayerPositionAndRotationPacketEvent>()
            .add_event::<PlayerSessionPacketEvent>()
            .add_event::<ChatCommandPacketEvent>()
            .add_event::<ChatMessagePacketEvent>()
            .add_event::<SwingArmPacketEvent>();
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PacketHandler;
