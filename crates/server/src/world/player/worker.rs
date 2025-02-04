use std::any::Any;
use std::fs::File;
use std::io::BufReader;

use bevy_ecs::prelude::Resource;
use flume::{Receiver, Sender};

use spherix_config::Config;

use crate::entities::living::player::{LoadPropertiesTask, LoadPropertiesTaskResult};
use crate::perf::worker::DynamicTaskHandler;
use crate::world::player::properties::Properties;
use spherix_world::io::Compression;

#[derive(Resource)]
pub struct LoadPropertiesTaskResultReceiver(pub Receiver<LoadPropertiesTaskResult>);

pub struct LoadPropertiesTaskHandler {
    pub config: Config,
    pub result_tx: Sender<LoadPropertiesTaskResult>,
}

impl DynamicTaskHandler for LoadPropertiesTaskHandler {
    fn handle(&self, task: Box<dyn Any>) {
        let boxed_task = task.downcast::<LoadPropertiesTask>().unwrap();

        let playerdata_path = self.config.world.path.clone()
            .inner()
            .join("playerdata")
            .join("example2.nbt");
        let playerdata_file = File::open(playerdata_path.clone()).unwrap();

        let prop = Properties::read(&mut BufReader::new(playerdata_file), Some(Compression::Gzip)).unwrap();

        let mut playerdata_file = std::fs::OpenOptions::new()
            .write(true)
            .open(playerdata_path)
            .unwrap();

        prop.write(&mut playerdata_file, Some(Compression::Gzip)).unwrap();

        self.result_tx
            .send(LoadPropertiesTaskResult {
                client: boxed_task.client,
                properties: prop,
            })
            .unwrap()
    }
}
