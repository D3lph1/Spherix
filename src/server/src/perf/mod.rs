use std::any::Any;

use bevy_ecs::prelude::Resource;
use flume::Sender;

pub mod worker;

#[derive(Resource)]
pub struct GeneralPurposeTaskSender(pub Sender<Box<dyn Any + Send>>);
