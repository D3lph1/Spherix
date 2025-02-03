use std::fmt::Debug;
use std::time::Duration;

use flume::{Receiver, Sender};
use owo_colors::OwoColorize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::time::timeout;
use tracing::trace;

use spherix_proto::codec::{ReadableCodec, WritableCodec};
use spherix_proto::io::Error;
use spherix_proto::io::{Readable, Writable};
use spherix_proto::packet::clientbound::PlayMapping as ClientboundPlayMapping;
use spherix_proto::packet::serverbound::PlayMapping as ServerboundPlayMapping;

const READER_BUF_SIZE: usize = 512;

pub struct Reader {
    conn_id: i32,
    stream: OwnedReadHalf,
    codec: ReadableCodec,
    buf: [u8; READER_BUF_SIZE],
    received: Sender<ServerboundPlayMapping>
}

impl Reader {
    pub fn new(conn_id: i32, stream: OwnedReadHalf, codec: ReadableCodec, received: Sender<ServerboundPlayMapping>) -> Self {
        Reader {
            conn_id,
            stream,
            codec,
            buf: [0; READER_BUF_SIZE],
            received
        }
    }

    pub async fn receive(&mut self) -> Result<(), Error> {
        let read = self.stream.read(&mut self.buf).await?;
        self.codec.append(&self.buf[..read]);

        Ok(())
    }

    pub async fn read<R: Readable + Debug>(&mut self) -> Result<R, Error> {
        loop {
            if let Some(packet) = self.codec.next()? {
                trace!("{}  {:?}", format!("[{}]", format!("< {}", self.conn_id).bright_black()).on_yellow(), packet);

                return Ok(packet)
            }

            if let Ok(read) = timeout(Duration::from_secs(5), self.stream.read(&mut self.buf)).await {
                let read = read?;

                if read == 0 {
                    return Err(Error::Eof)
                }

                self.codec.append(&self.buf[..read]);
            } else {
                return Err(Error::Timeout)
            }
        }
    }

    pub async fn work(mut self) -> Result<(), Error> {
        loop {
            let packet = self.read::<ServerboundPlayMapping>().await?;

            if self.received.send_async(packet).await.is_err() {
                return Ok(());
            }
        }
    }

    pub fn stream(&mut self) -> &mut OwnedReadHalf {
        &mut self.stream
    }

    pub fn codec(&mut self) -> &mut ReadableCodec {
        return &mut self.codec
    }
}

pub struct Writer {
    conn_id: i32,
    stream: OwnedWriteHalf,
    codec: WritableCodec,
    buf: Vec<u8>,
    to_send: Receiver<ClientboundPlayMapping>
}

impl Writer {
    pub fn new(conn_id: i32, stream: OwnedWriteHalf, codec: WritableCodec, to_send: Receiver<ClientboundPlayMapping>) -> Self {
        Self {
            conn_id,
            stream,
            codec,
            buf: Vec::new(),
            to_send
        }
    }

    pub async fn write<W: Writable + Debug>(&mut self, packet: W) -> Result<(), Error> {
        trace!("{} {:?}", format!("[{}]", format!("> {}", self.conn_id).bright_black()).on_red(), packet);
        self.codec.write(&packet, &mut self.buf)?;
        self.stream.write(&mut self.buf).await?;
        self.stream.flush().await?;
        self.buf.clear();

        Ok(())
    }

    pub async fn work(mut self) -> Result<(), Error> {
        loop {
            if let Ok(packet) = self.to_send.recv_async().await {
                if let ClientboundPlayMapping::Disconnect(packet) = packet {
                    self.write(ClientboundPlayMapping::Disconnect(packet)).await?;

                    return Ok(())
                } else {
                    self.write(packet).await?;
                }
            } else {
                return Err(Error::Eof)
            }
        }
    }

    pub fn stream(&mut self) -> &mut OwnedWriteHalf {
        &mut self.stream
    }

    pub fn codec(&mut self) -> &mut WritableCodec {
        &mut self.codec
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        trace!("Connection closed");
    }
}
