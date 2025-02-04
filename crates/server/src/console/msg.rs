use bevy_ecs::prelude::{Entity, Resource};
use flume::Receiver;

pub enum CommandSource {
    Console,
    Player(Entity),
}

pub struct Command {
    pub source: CommandSource,
    pub text: String,
}

#[derive(Resource)]
pub struct CommandReceiver(pub Receiver<Command>);
