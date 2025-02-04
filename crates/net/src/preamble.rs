use std::io::Cursor;

use anyhow::anyhow;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use spherix_config::Config;
use spherix_proto::packet::clientbound::{PingResponse, StatusMapping as ClientboundStatusMapping, StatusResponse};
use spherix_proto::packet::serverbound::{HandshakingMapping, StatusMapping as ServerboundStatusMapping};

use crate::auth::handle_login;
use crate::conn::Connection;
use crate::io::{Reader, Writer};
use crate::join::JoinContext;

pub struct Preamble<'a> {
    pub conn: &'a mut Connection,
    pub reader: &'a mut Reader,
    pub writer: &'a mut Writer,
    biomes: nbt::Value,
}

#[derive(Debug)]
pub enum PreambleError {
    Handshaking(anyhow::Error),
    Status(anyhow::Error),
    Login(anyhow::Error),
}

impl<'a> Preamble<'a> {
    pub fn new(conn: &'a mut Connection, reader: &'a mut Reader, writer: &'a mut Writer, biomes: nbt::Value) -> Self {
        Self {
            conn,
            reader,
            writer,
            biomes,
        }
    }

    pub fn to_join_context(self, name: String, uuid: Uuid) -> JoinContext<'a> {
        JoinContext::new(name, uuid, self.conn, self.reader, self.writer, self.biomes)
    }
}

pub async fn handle_preamble(preamble: Preamble<'_>, config: Config) -> Result<Option<JoinContext<'_>>, PreambleError> {
    preamble.reader.receive().await.unwrap();
    let mut cursor = Cursor::new(preamble.reader.codec().buf());

    // Legacy ping
    if cursor.read_u8().await.unwrap() == 0xFE {
        handle_legacy_ping(&mut cursor, preamble.writer)
            .await
            .map_err(|e| PreambleError::Handshaking(e))?;

        return Ok(None);
    }

    let h = preamble.reader.read::<HandshakingMapping>()
        .await
        .map_err(|e| PreambleError::Handshaking(anyhow!(e)))?;

    let HandshakingMapping::Handshake(h) = h;

    return if h.next_state == 1 {
        handle_status(preamble)
            .await
            .map_err(|e| PreambleError::Status(e))?;

        Ok(None)
    } else {
        let res = handle_login(preamble, &config.clone())
            .await
            .map_err(|e| PreambleError::Login(e));

        Ok(Some(res?))
    };
}

/// The method provides compatibility with outdated Minecraft clients by
/// notifying them that it is not possible to connect to this server
async fn handle_legacy_ping(cursor: &mut Cursor<&Vec<u8>>, writer: &mut Writer) -> anyhow::Result<()> {
    // The following code cares only about "marker" bytes and ignores a subsequent payload
    // as suggested in the protocol docs: https://wiki.vg/Server_List_Ping#1.6

    let mb_01 = cursor.read_u8().await?;
    if mb_01 != 0x01 {
        return Err(anyhow!("expected 0x01 as the first byte for legacy ping, but {:#04x} given", mb_01));
    }

    // Handle 0xFE 0x01 0xFA or 0xFE 0x01 ping
    Ok(write_legacy_ping_response(writer).await?)
}

async fn write_legacy_ping_response(writer: &mut Writer) -> anyhow::Result<()> {
    let str: Vec<u16> = "§1\047\01.4.2\0Hello World\05\0100".encode_utf16().collect();
    let u16_bytes = str.as_slice();
    // String length in bytes. We have to increment value by 1 for supporting odd-sized strings.
    // --------------------------------------------------------- \/ -----------------
    let mut u8_bytes = vec![0u8; u16_bytes.len() * 2 + 1].into_boxed_slice();

    u16_to_u8_cpy(u16_bytes, &mut u8_bytes);

    // Kick packet
    writer.stream().write_u8(0xFF).await?;
    // String length in bytes
    writer.stream().write_u16(str.len() as u16).await?;
    // String itself
    writer.stream().write(u8_bytes.as_ref()).await?;

    writer.stream().flush().await?;

    Ok(())
}

pub fn u16_to_u8_cpy(bytes: &[u16], my_bytes: &mut [u8]) {
    bytes.iter()
        .zip(my_bytes.chunks_exact_mut(2))
        .for_each(|(a, b)| b.copy_from_slice(&a.to_be_bytes()));
}

async fn handle_status(preamble: Preamble<'_>) -> anyhow::Result<()> {
    let req = preamble.reader.read::<ServerboundStatusMapping>().await?;

    let ServerboundStatusMapping::StatusRequest(_) = req else {
        return Err(anyhow!("expected StatusRequest packet, but {} given", req.name()));
    };

    let json = r#"{"version":{"name":"1.19.4","protocol":762},"players":{"max":100,"online":5,"sample":[{"name":"D3lph1","id":"4566e69f-c907-48ee-8d71-d7ba5aa00d20"}]},"description":{"text":"Hello world"},"enforcesSecureChat":true}"#.to_string();

    preamble.writer.write(ClientboundStatusMapping::StatusResponse(StatusResponse {
        json,
    })).await?;

    let ping = preamble.reader.read::<ServerboundStatusMapping>().await.unwrap();

    let ServerboundStatusMapping::PingRequest(ping) = ping else {
        return Err(anyhow!("expected PingRequest packet, but {} given", ping.name()));
    };

    preamble.writer.write(ClientboundStatusMapping::PingResponse(PingResponse {
        payload: ping.payload
    })).await?;

    Ok(())
}
