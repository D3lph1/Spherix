use std::net::SocketAddr;
use std::sync::atomic::{AtomicI32, Ordering};
use std::sync::Arc;

use flume::Sender;
use owo_colors::OwoColorize;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_util::sync::CancellationToken;
use tracing::info;

use spherix_config::Config;

use crate::client::Client;
use crate::conn::Connection;
use crate::worker::Worker;

pub struct NetServer {
    next_id: AtomicI32,
    addr: SocketAddr,
    cancel: CancellationToken,
    players: Sender<Client>,
    biomes: nbt::Value,
    config: Config
}

impl NetServer {
    pub fn new(
        addr: SocketAddr,
        cancel: CancellationToken,
        players: Sender<Client>,
        biomes: nbt::Value,
        config: Config
    ) -> Self {
        Self {
            next_id: AtomicI32::new(0),
            addr,
            cancel,
            players,
            biomes,
            config
        }
    }

    pub async fn serve(self) {
        Arc::new(self).listen().await;
    }

    async fn listen(self: Arc<Self>) {
        let listener = TcpListener::bind(self.addr).await.unwrap();
        info!(
            "Listening socket {} for incoming connections...",
            format!(
                "{}:{}",
                 self.addr.ip(),
                 self.addr.port()
            ).cyan().underline()
        );

        loop {
            let cancel = self.cancel.clone();

            select! {
                _ = cancel.cancelled() => {
                    return;
                }
                res = listener.accept() => {
                    let (stream, addr) = res.unwrap();

                    tokio::spawn({
                        let this = Arc::clone(&self);

                        async move {
                            this.accept(stream, addr);
                        }
                    });
                }
            }
        }
    }

    fn accept(&self, stream: TcpStream, addr: SocketAddr) {
        let conn = Connection::new(self.next_id.fetch_add(1, Ordering::Relaxed), addr);

        let worker = Worker::new(conn, stream, self.players.clone(), self.biomes.clone());

        worker.start(self.config.clone());
    }
}
