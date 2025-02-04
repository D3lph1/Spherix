use std::io::{Read, Write};

use uuid::Uuid;

use crate::io::{BitSet, Byte, Error, Long, Readable, VarInt, Writable};
use crate::packet::{packet, packet_wo_io};

const MESSAGE_SIGNATURE_LENGTH: usize = 256;
pub type MessageSignature = Box<[Byte; MESSAGE_SIGNATURE_LENGTH]>;

packet_wo_io!(
    PlayerChatMessage {
        sender: Uuid,
        index: VarInt,
        message_signature: Option<MessageSignature>,
        body: SignedMessageBody,
        unsigned_content: Option<String>,
        filter_type: VarInt,
        filter_type_bits: Option<BitSet>,
        chat_type: VarInt,
        network_name: String,
        network_target_name: Option<String>
    }
);

impl Readable for PlayerChatMessage {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let sender = Uuid::read(buf)?;
        let index = VarInt::read(buf)?;
        let message_signature = Option::read(buf)?;
        let body = SignedMessageBody::read(buf)?;
        let unsigned_content = Option::read(buf)?;
        let filter_type = VarInt::read(buf)?;
        let filter_type_bits = if filter_type == VarInt(2) {
            Some(BitSet::read(buf)?)
        } else {
            None
        };
        let chat_type = VarInt::read(buf)?;
        let network_name = String::read(buf)?;
        let network_target_name = Option::read(buf)?;

        Ok(Self {
            sender,
            index,
            message_signature,
            body,
            unsigned_content,
            filter_type,
            filter_type_bits,
            chat_type,
            network_name,
            network_target_name,
        })
    }
}

impl Writable for PlayerChatMessage {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let mut written = self.sender.write(buf)?;
        written += self.index.write(buf)?;
        written += self.message_signature.write(buf)?;
        written += self.body.write(buf)?;
        written += self.unsigned_content.write(buf)?;
        written += self.filter_type.write(buf)?;
        if self.filter_type == VarInt(2) {
            written += self.filter_type_bits.write(buf)?;
        }
        written += self.chat_type.write(buf)?;
        written += self.network_name.write(buf)?;
        written += self.network_target_name.write(buf)?;

        Ok(written)
    }
}

packet!(
    SignedMessageBody {
        content: String,
        timestamp: Long,
        salt: Long,
        last_seen_messages: LastSeenMessages
    }
);

packet!(
    LastSeenMessages {
        entries: Vec<LastSeenSignature>
    }
);

packet_wo_io!(
    LastSeenSignature {
        id: VarInt,
        signature: Option<MessageSignature>
    }
);

impl From<MessageSignature> for LastSeenSignature {
    fn from(value: MessageSignature) -> Self {
        Self {
            id: VarInt(-1),
            signature: Some(value)
        }
    }
}

impl From<VarInt> for LastSeenSignature {
    fn from(value: VarInt) -> Self {
        Self {
            id: value,
            signature: None,
        }
    }
}

impl Readable for LastSeenSignature {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let id = VarInt::read(buf)? - 1;

        if id == -1 {
            return Ok(Self::from(MessageSignature::read(buf)?))
        }

        Ok(Self::from(id))
    }
}

impl Writable for LastSeenSignature {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        // Type suffix (1i32) is required for compiler to understand what
        // implementation of std::ops::Add trait to call (in this case
        // will be called impl Add<i32> for VarInt).
        let mut written = (self.id + 1i32).write(buf)?;
        if self.signature.is_some() {
            written += self.signature.as_ref().unwrap().write(buf)?;
        }

        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;

    use crate::packet::clientbound::*;
    use crate::packet::tests::synthetic_test;

    #[test]
    fn player_chat_message() {
        synthetic_test::<PlayerChatMessage>();
    }

    #[test]
    fn signed_message_body() {
        synthetic_test::<SignedMessageBody>();
    }

    #[test]
    fn last_seen_messages() {
        synthetic_test::<LastSeenMessages>();
    }

    #[test]
    fn last_seen_signature() {
        synthetic_test::<LastSeenSignature>();
    }
}
