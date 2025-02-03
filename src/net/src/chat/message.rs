use std::time::{Duration, Instant};

use spherix_proto::io::{Long, VarInt};
use spherix_proto::packet::clientbound::{LastSeenMessages as PackedLastSeenMessages, LastSeenSignature, MessageSignature, SignedMessageBody};
use spherix_util::time::{instant_from_millis, is_after};

use crate::chat::MessageSignatureCache;

#[derive(Debug, Clone)]
pub struct UnpackedSignedMessageBody {
    pub content: String,
    pub salt: Long,
    pub timestamp: Long,
    pub last_seen_messages: UnpackedLastSeenMessages,
}

impl UnpackedSignedMessageBody {
    pub fn pack(&self, message_signature_cache: &MessageSignatureCache) -> SignedMessageBody {
        SignedMessageBody {
            content: self.content.clone(),
            salt: self.salt,
            timestamp: self.timestamp,
            last_seen_messages: self.last_seen_messages.pack(message_signature_cache),
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnpackedMessageSignature(pub MessageSignature);

impl UnpackedMessageSignature {
    fn pack(&self, signature_cache: &MessageSignatureCache) -> LastSeenSignature {
        let id = signature_cache.pack(&self.0);

        if id != -1 {
            return LastSeenSignature::from(VarInt(id));
        }

        LastSeenSignature::from(self.0.clone())
    }
}

#[derive(Debug, Clone)]
pub struct UnpackedLastSeenMessages(pub Vec<UnpackedMessageSignature>);

impl UnpackedLastSeenMessages {
    fn pack(&self, message_signature_cache: &MessageSignatureCache) -> PackedLastSeenMessages {
        PackedLastSeenMessages {
            entries: self.0
                .iter()
                .map(|last_seen| {
                    last_seen.pack(message_signature_cache)
                })
                .collect()
        }
    }
}

#[derive(Debug, Clone)]
pub struct UnpackedPlayerChatMessage {
    pub signature: Option<MessageSignature>,
    pub body: UnpackedSignedMessageBody,
}

impl UnpackedPlayerChatMessage {
    const MESSAGE_EXPIRES_AFTER_SERVER: Duration = Duration::from_secs(5 * 60); // 5 minutes

    pub fn is_chat_trusted(&self) -> bool {
        self.signature.is_some() && self.has_expired_server(Instant::now())
    }

    fn has_expired_server(&self, at: Instant) -> bool {
        is_after(
            at,
            instant_from_millis(self.body.timestamp as u64) + Self::MESSAGE_EXPIRES_AFTER_SERVER
        )
    }
}
