use std::time::Duration;

use flume::{Receiver, Sender};
use tokio::net::TcpStream;
use tokio::task::JoinSet;
use tokio::time::sleep;
use tracing::error;

use spherix_config::Config;
use spherix_proto::codec::{ReadableCodec, WritableCodec};
use spherix_proto::io::Error;
use spherix_proto::packet::clientbound::{LoginDisconnect, LoginMapping, PlayMapping as ClientboundPlayMapping};
use spherix_proto::packet::serverbound::PlayMapping as ServerboundPlayMapping;

use crate::client::Client;
use crate::conn::Connection;
use crate::io::{Reader, Writer};
use crate::join::join;
use crate::preamble::{handle_preamble, Preamble, PreambleError};

pub struct Worker {
    conn: Connection,
    reader: Reader,
    writer: Writer,
    received: Receiver<ServerboundPlayMapping>,
    to_send: Sender<ClientboundPlayMapping>,
    players: Sender<Client>,
    biomes: nbt::Value
}

impl Worker {
    pub fn new(conn: Connection, stream: TcpStream, players: Sender<Client>, biomes: nbt::Value) -> Self {
        let (reader, writer) = stream.into_split();
        let (received_tx, received_rx) = flume::bounded::<ServerboundPlayMapping>(32);
        let (to_send_tx, to_send_rx) = flume::unbounded::<ClientboundPlayMapping>();

        let conn_id = conn.id;

        Self {
            conn,
            reader: Reader::new(conn_id, reader, ReadableCodec::new(), received_tx),
            writer: Writer::new(conn_id, writer, WritableCodec::new(), to_send_rx),
            received: received_rx,
            to_send: to_send_tx,
            players,
            biomes
        }
    }

    pub fn start(mut self, config: Config) {
        tokio::task::spawn(async move {
            let preamble = Preamble::new(&mut self.conn, &mut self.reader, &mut self.writer, self.biomes);

            let join_ctx = handle_preamble(preamble, config).await;

            if join_ctx.is_err() {
                let err = join_ctx.err().unwrap();
                let (msg, err) = match err {
                    PreambleError::Handshaking(e) => {
                        (format!("Error during handshake: {}", e), e)
                    }
                    PreambleError::Status(e) => {
                        (format!("Error during status: {}", e), e)
                    }
                    PreambleError::Login(e) => {
                        (format!("Error during login: {}", e), e)
                    }
                };

                self.writer
                    .write(LoginMapping::LoginDisconnect(LoginDisconnect {
                        reason: format!("{{\"text\": \"{}\"}}", err).to_string(),
                    }))
                    .await
                    .unwrap();

                error!("{msg}");

                return;
            }

            let join_ctx_val = join_ctx.unwrap();
            if join_ctx_val.is_none() {
                return;
            }

            let join_ctx = join_ctx_val.unwrap();
            let client = join_ctx.to_client(self.received.clone(), self.to_send.clone());

            join(join_ctx).await;

            self
                .players
                .send_async(client)
                .await
                .unwrap();

            let reader = self.reader;
            let writer = self.writer;

            let mut join_set = JoinSet::new();

            join_set.spawn(async move {
                let res = reader.work().await;

                match res {
                    Ok(_) => {}
                    Err(err) => match err {
                        Error::Eof => {}
                        err => panic!("{:?}", err)
                    }
                }
            });

            join_set.spawn(async move {
                let _ = writer.work().await;
            });

            while let Some(_) = join_set.join_next().await {
                return;
            }
        });
    }
}
