use uuid::Uuid;

use crate::io::{bitset_bits_to_bytes, BitSet, Byte, ByteArray, Double, FixedBitSet, Float, Long, Position, Short, UnsignedByte, UnsignedShort, VarInt};
use crate::packet::{packet, packet_serverbound};

packet!(
    Handshake {
        protocol_version: VarInt,
        server_address: String,
        server_port: UnsignedShort,
        next_state: VarInt
    }
);

packet_serverbound!(
    HandshakingMapping {
        0x00 = Handshake
    }
);

packet!(
    StatusRequest {}

    PingRequest {
        payload: Long
    }
);

packet_serverbound!(
    StatusMapping {
        0x00 = StatusRequest,
        0x01 = PingRequest
    }
);

packet!(
    LoginStart {
        name: String,
        has_player_uuid: bool,
        player_uuid: Uuid
    }

    EncryptionResponse {
        shared_secret_key: Box<[UnsignedByte]>,
        verify_token: Box<[UnsignedByte]>
    }
);

packet_serverbound!(
    LoginMapping {
        0x00 = LoginStart,
        0x01 = EncryptionResponse
    }
);

packet!(
    ConfirmTeleportation {
        teleport_id: VarInt
    }

    ChatCommand {
        message: String,
        timestamp: Long,
        salt: Long,
        argument_signatures: Vec<ChatCommandArgumentSignature>,
        message_count: VarInt,
        acknowledged: BitSet
    }

    // Component of the previous packet, not a packet itself
    ChatCommandArgumentSignature {
        name: String,
        signature: Box<[Byte]>
    }

    ChatMessage {
        message: String,
        timestamp: Long,
        salt: Long,
        signature: Option<Box<[Byte; 256]>>,
        message_count: VarInt,
        acknowledged: FixedBitSet<{bitset_bits_to_bytes(20)}>
    }

    PlayerSession {
        session_id: Uuid,
        expires_at: Long,
        public_key: Box<[Byte]>,
        key_signature: Box<[Byte]>
    }

    ClientInformation {
        locale: String,
        view_distance: Byte,
        chat_mode: VarInt,
        chat_colors: bool,
        displayed_skin_parts: UnsignedByte,
        main_hand: VarInt,
        enable_text_filtering: bool,
        allow_server_listing: bool
    }

    PluginMessage {
        channel: String,
        data: ByteArray
    }

    SetPlayerPosition {
        x: Double,
        feet_y: Double,
        z: Double,
        on_ground: bool
    }

    SetPlayerPositionAndRotation {
        x: Double,
        feet_y: Double,
        z: Double,
        yaw: Float,
        pitch: Float,
        on_ground: bool
    }

    SetPlayerRotation {
        yaw: Float,
        pitch: Float,
        on_ground: bool
    }

    KeepAlive {
        keep_alive_id: Long
    }

    SetPlayerOnGround {
        on_ground: bool
    }

    PlayerAbilities {
        flag: Byte
    }

    PlayerAction {
        status: VarInt,
        location: Position,
        face: Byte,
        sequence: VarInt
    }

    PlayerCommand {
        entity_id: VarInt,
        action_id: VarInt,
        jump_boost: VarInt
    }

    SetHeldItem {
        slot: Short
    }

    SwingArm {
        hand: VarInt
    }
);

packet_serverbound!(
    PlayMapping {
        0x00 = ConfirmTeleportation,
        0x04 = ChatCommand,
        0x05 = ChatMessage,
        0x06 = PlayerSession,
        0x08 = ClientInformation,
        0x0D = PluginMessage,
        0x12 = KeepAlive,
        0x14 = SetPlayerPosition,
        0x15 = SetPlayerPositionAndRotation,
        0x16 = SetPlayerRotation,
        0x17 = SetPlayerOnGround,
        0x1C = PlayerAbilities,
        0x1D = PlayerAction,
        0x1E = PlayerCommand,
        0x28 = SetHeldItem,
        0x2F = SwingArm
    }
);
