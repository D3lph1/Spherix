use std::collections::HashMap;
use std::io::{Read, Write};

use anyhow::anyhow;
use nbt::Blob;
use uuid::Uuid;

use crate::io::*;
pub use crate::packet::clientbound::chat::*;
pub use crate::packet::clientbound::player_info_update::*;
use crate::packet::{packet, packet_clientbound};

mod chat;
mod player_info_update;

packet!(
    StatusResponse {
        json: String
    }

    PingResponse {
        payload: Long
    }
);

packet_clientbound!(
    StatusMapping {
        0x00 = StatusResponse,
        0x01 = PingResponse
    }
);

packet!(
    LoginDisconnect {
        reason: String
    }

    EncryptionRequest {
        server_id: String,
        public_key: Box<[UnsignedByte]>,
        verify_token: Box<[UnsignedByte]>
    }

    LoginSuccess {
        uuid: Uuid,
        username: String,
        properties: Box<[LoginSuccessProperty]>
    }

    LoginSuccessProperty {
        name: String,
        value: String,
        signature: Option<String>
    }

    SetCompression {
        threshold: VarInt
    }
);

packet_clientbound!(
    LoginMapping {
        0x00 = LoginDisconnect,
        0x01 = EncryptionRequest,
        0x02 = LoginSuccess,
        0x03 = SetCompression
    }
);

packet!(
    BundleDelimiter {}

    SpawnEntity {
        entity_id: VarInt,
        entity_uuid: Uuid,
        entity_type: VarInt,
        x: Double,
        y: Double,
        z: Double,
        pitch: Angle,
        yaw: Angle,
        head_yaw: Angle,
        data: VarInt,
        velocaity_x: Short,
        velocaity_y: Short,
        velocaity_z: Short
    }

    SpawnPlayer {
        entity_id: VarInt,
        player_uuid: Uuid,
        x: Double,
        y: Double,
        z: Double,
        yaw: Angle,
        pitch: Angle
    }

    EntityAnimation {
        entity_id: VarInt,
        animation: Byte
    }

    UnloadChunk {
        x: Int,
        z: Int
    }

    InitializeWorldBorder {
        x: Double,
        z: Double,
        old_diameter: Double,
        new_diameter: Double,
        speed: VarLong,
        portal_teleport_boundary: VarInt,
        warning_blocks: VarInt,
        warning_time: VarInt
    }

    Login {
        entity_id: Int,
        is_hardcore: bool,
        gamemode: UnsignedByte,
        previous_gamemode: Byte,
        dimensions: Box<[String]>,
        registry_codec: Blob,
        dimension_type: String,
        dimension_name: String,
        hashed_seed: Long,
        max_players: VarInt,
        view_distance: VarInt,
        simulation_distance: VarInt,
        reduced_debug_info: bool,
        enable_respawn_screen: bool,
        is_debug: bool,
        is_flat: bool,
        has_death_location: bool
    }

    UpdateEntityPosition {
        entity_id: VarInt,
        delta_x: Short,
        delta_y: Short,
        delta_z: Short,
        on_ground: bool
    }

    UpdateEntityPositionAndRotation {
        entity_id: VarInt,
        delta_x: Short,
        delta_y: Short,
        delta_z: Short,
        yaw: Angle,
        pitch: Angle,
        on_ground: bool
    }

    UpdateEntityRotation {
        entity_id: VarInt,
        yaw: Angle,
        pitch: Angle,
        on_ground: bool
    }

    PlayerInfoRemove {
        players: Vec<Uuid>
    }

    SynchronizePlayerPosition {
        x: Double,
        y: Double,
        z: Double,
        yaw: Float,
        pitch: Float,
        flags: Byte,
        teleport_id: VarInt
    }

    RemoveEntities {
        entity_ids: Vec<VarInt>
    }

    SetHeadRotation {
        entity_id: VarInt,
        head_yaw: Angle
    }

    ServerData {
        motd: String,
        icon: Option<String>,
        enforces_secure_chat: bool
    }

    SetCenterChunk {
        chunk_x: VarInt,
        chunk_z: VarInt
    }

    SetDefaultSpawnPosition {
        location: Position,
        angle: Float
    }

    ChunkData {
        chunk_x: Int,
        chunk_z: Int,
        heightmaps: Blob,
        data: Box<[u8]>,
        number_of_block_entities: VarInt,
        trust_edges: bool,
        sky_light_mask: BitSet,
        block_light_mask: BitSet,
        empty_sky_light_mask: BitSet,
        empty_block_light_mask: BitSet,
        sky_light_arrays: Vec<Box<[u8]>>,
        block_light_arrays: Vec<Box<[u8]>>
    }

    SetHealth {
        health: Float,
        food: VarInt,
        food_saturation: Float
    }

    TeleportEntity {
        entity_id: VarInt,
        x: Double,
        y: Double,
        z: Double,
        yaw: Angle,
        pitch: Angle,
        on_ground: bool
    }

    KeepAlive {
        keep_alive_id: Long
    }

    Disconnect {
        reason: String
    }
);

packet_clientbound!(
    PlayMapping {
        0x00 = BundleDelimiter,
        0x01 = SpawnEntity,
        0x03 = SpawnPlayer,
        0x04 = EntityAnimation,
        0x1E = UnloadChunk,
        0x22 = InitializeWorldBorder,
        0x24 = ChunkData,
        0x28 = Login,
        0x2B = UpdateEntityPosition,
        0x2C = UpdateEntityPositionAndRotation,
        0x2D = UpdateEntityRotation,
        0x35 = PlayerChatMessage,
        0x39 = PlayerInfoRemove,
        0x3A = PlayerInfoUpdate,
        0x3C = SynchronizePlayerPosition,
        0x3E = RemoveEntities,
        0x42 = SetHeadRotation,
        0x45 = ServerData,
        0x4E = SetCenterChunk,
        0x50 = SetDefaultSpawnPosition,
        0x57 = SetHealth,
        0x68 = TeleportEntity,
        0x23 = KeepAlive,
        0x1A = Disconnect
    }
);

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use spherix_util::hex::decode_stringed_hex;

    use crate::io::VarInt;
    use crate::io::{Readable, Writable};
    use crate::packet::clientbound::PlayerChatMessage;

    const HEX: &str = r"
        35 5E 60 83 74 86 92 31 CB 98 11 FA
        6C 6C 02 0E 75 00 00 05 66 69 72 73
        74 00 00 01 8E B4 31 5F 36 00 00 00
        00 00 00 00 00 00 00 00 00 FA 01 7B
        22 69 6E 73 65 72 74 69 6F 6E 22 3A
        22 44 33 6C 70 68 31 22 2C 22 63 6C
        69 63 6B 45 76 65 6E 74 22 3A 7B 22
        61 63 74 69 6F 6E 22 3A 22 73 75 67
        67 65 73 74 5F 63 6F 6D 6D 61 6E 64
        22 2C 22 76 61 6C 75 65 22 3A 22 2F
        74 65 6C 6C 20 44 33 6C 70 68 31 20
        22 7D 2C 22 68 6F 76 65 72 45 76 65
        6E 74 22 3A 7B 22 61 63 74 69 6F 6E
        22 3A 22 73 68 6F 77 5F 65 6E 74 69
        74 79 22 2C 22 63 6F 6E 74 65 6E 74
        73 22 3A 7B 22 74 79 70 65 22 3A 22
        6D 69 6E 65 63 72 61 66 74 3A 70 6C
        61 79 65 72 22 2C 22 69 64 22 3A 22
        35 65 36 30 38 33 37 34 2D 38 36 39
        32 2D 33 31 63 62 2D 39 38 31 31 2D
        66 61 36 63 36 63 30 32 30 65 37 35
        22 2C 22 6E 61 6D 65 22 3A 7B 22 74
        65 78 74 22 3A 22 44 33 6C 70 68 31
        22 7D 7D 7D 2C 22 74 65 78 74 22 3A
        22 44 33 6C 70 68 31 22 7D 00
    ";

    #[test]
    fn test() {
        let mut src_buf = Cursor::new(decode_stringed_hex(HEX).unwrap());

        let _id = VarInt::read(&mut src_buf).unwrap();
        let read_packet = PlayerChatMessage::read(&mut src_buf).unwrap();

        let mut dst_buf = Vec::new();

        VarInt(0x35).write(&mut dst_buf).unwrap();
        read_packet.write(&mut dst_buf).unwrap();

        assert_eq!(src_buf.into_inner(), dst_buf);
    }
}
