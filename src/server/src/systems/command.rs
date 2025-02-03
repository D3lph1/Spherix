use bevy_ecs::prelude::{Event, EventReader, EventWriter, Res};
use tracing::debug;

use crate::console::msg::{Command, CommandReceiver, CommandSource};
use crate::systems::packet::ChatCommandPacketEvent;

#[derive(Event)]
pub struct ChatCommandEvent(pub Command);

pub fn on_chat_command_packet(
    mut rx: EventReader<ChatCommandPacketEvent>,
    mut tx: EventWriter<ChatCommandEvent>
) {
    for event in rx.read() {
        tx.send(ChatCommandEvent(Command {
            source: CommandSource::Player(event.entity),
            text: event.packet.message.clone(),
        }));
    }
}

pub fn on_command(
    commands: Res<CommandReceiver>,
    mut tx: EventWriter<ChatCommandEvent>
) {
    for command in commands.0.try_iter() {
        tx.send(ChatCommandEvent(command));
    }
}

pub fn poll_commands(
    mut rx: EventReader<ChatCommandEvent>
) {
    for event in rx.read() {
        debug!("{}", event.0.text);
    }
}
