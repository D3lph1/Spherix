use bevy_ecs::prelude::Resource;
use flume::Receiver;

use spherix_net::client::Client;
use spherix_net::server::NetServer;

#[derive(Resource)]
pub struct Server {

}

impl Server {
    pub async fn start(net: NetServer) -> Self {
        tokio::task::spawn(async move {
            net.serve().await
        });

        let server = Self {

        };

        server
    }
}

#[derive(Resource)]
pub struct ClientReceiver(pub Receiver<Client>);
