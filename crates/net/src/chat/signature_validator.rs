use spherix_proto::packet::clientbound::MessageSignature;
use spherix_proto::packet::serverbound::ChatMessage;

use crate::chat::message::UnpackedMessageSignature;
use crate::chat::UnpackedLastSeenMessages;

pub struct LastSeenMessagesValidator {
    last_seen_count: usize,
    tracked_messages: Vec<Option<LastSeenTrackedEntry>>,
    last_pending_message: Option<MessageSignature>
}

impl LastSeenMessagesValidator {
    pub fn new(size: usize) -> Self {
        let mut tracked_messages = Vec::with_capacity(size);
        for _ in 0..size {
            tracked_messages.push(None);
        }

        Self {
            last_seen_count: size,
            tracked_messages,
            last_pending_message: None
        }
    }

    pub fn add_pending(&mut self, signature: MessageSignature) {
        if Some(&signature) != self.last_pending_message.as_ref() {
            self.tracked_messages.push(Some(LastSeenTrackedEntry {
                signature: signature.clone(),
                pending: true,
            }));

            self.last_pending_message = Some(signature)
        }
    }

    fn tracked_messages_count(&self) -> usize {
        self.tracked_messages.len()
    }

    fn apply_offset(&mut self, offset: usize) -> bool {
        let i = self.tracked_messages.len() - self.last_seen_count;
        if offset >= 0 && offset <= i {
            // remove first `offset` elements from `self.tracked_messages`
            for _ in 0..offset {
                self.tracked_messages.remove(0);
            }

            return true;
        }

        return false;
    }

    pub fn apply_update(&mut self, chat_message: ChatMessage) -> Option<UnpackedLastSeenMessages> {
        if !self.apply_offset(chat_message.message_count.0 as usize) {
            return None;
        }

        let mut object_list = Vec::with_capacity(chat_message.acknowledged.cardinality());
        if chat_message.acknowledged.length() > self.last_seen_count {
            return None
        }

        for i in 0..self.last_seen_count {
            let is_bit_set = chat_message.acknowledged.get(i).unwrap();
            let last_seen_tracked_entry = self.tracked_messages.get(i).as_ref().unwrap().clone();
            if is_bit_set {
                if last_seen_tracked_entry.is_none() {
                    return None
                }

                object_list.push(UnpackedMessageSignature(last_seen_tracked_entry.clone().unwrap().signature));

                let binding = Some(last_seen_tracked_entry.as_ref().unwrap().acknowledge());
                drop(std::mem::replace(&mut self.tracked_messages[i], binding));
            } else {
                if last_seen_tracked_entry.is_some() && !last_seen_tracked_entry.as_ref().unwrap().pending {
                    return None
                }

                drop(std::mem::replace(&mut self.tracked_messages[i], None));
            }
        }

        Some(UnpackedLastSeenMessages(object_list))
    }
}

#[derive(Clone)]
pub(crate) struct LastSeenTrackedEntry {
    signature: MessageSignature,
    pending: bool
}

impl LastSeenTrackedEntry {
    fn acknowledge(&self) -> Self {
        if self.pending {
            return Self {
                signature: self.signature.clone(),
                pending: false,
            };
        }

        return self.clone()
    }
}

#[cfg(test)]
mod tests {
    use spherix_proto::io::{Byte, FixedBitSet, VarInt};
    use spherix_proto::packet::clientbound::MessageSignature;
    use spherix_proto::packet::serverbound::ChatMessage;

    use crate::chat::LastSeenMessagesValidator;

    #[test]
    fn test() {
        let mut validator = LastSeenMessagesValidator::new(20);

        validator.apply_update(msg(0, vec![]));
        validator.add_pending(signature("1"));

        validator.apply_update(msg(1, vec![19]));
        validator.add_pending(signature("2"));

        validator.apply_update(msg(1, vec![18, 19]));
        validator.add_pending(signature("3"));

        validator.apply_update(msg(1, vec![17, 18, 19]));
        validator.add_pending(signature("4"));

        validator.apply_update(msg(1, vec![16, 17, 18, 19]));
        validator.add_pending(signature("5"));

        validator.apply_update(msg(1, vec![15, 16, 17, 18, 19]));
        validator.add_pending(signature("6"));

        validator.apply_update(msg(1, vec![14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("7"));

        validator.apply_update(msg(1, vec![13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("8"));

        validator.apply_update(msg(1, vec![12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("9"));

        validator.apply_update(msg(1, vec![11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("10"));

        validator.apply_update(msg(1, vec![10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("11"));

        validator.apply_update(msg(1, vec![9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("12"));

        validator.apply_update(msg(1, vec![8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("13"));

        validator.apply_update(msg(1, vec![7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("14"));

        validator.apply_update(msg(1, vec![6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("15"));

        validator.apply_update(msg(1, vec![5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("16"));

        validator.apply_update(msg(1, vec![4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("17"));

        validator.apply_update(msg(1, vec![3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("18"));

        validator.apply_update(msg(1, vec![2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("19"));

        validator.apply_update(msg(1, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("20"));

        validator.apply_update(msg(1, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("21"));

        validator.apply_update(msg(1, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("22"));

        validator.add_pending(signature("91"));

        validator.add_pending(signature("92"));

        validator.add_pending(signature("93"));

        validator.apply_update(msg(4, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19]));
        validator.add_pending(signature("23"));

        assert_eq!(21, validator.tracked_messages.len());
        assert_eq!(signature("6"), validator.tracked_messages[0].as_ref().unwrap().signature);
        assert_eq!(signature("7"), validator.tracked_messages[1].as_ref().unwrap().signature);
        assert_eq!(signature("8"), validator.tracked_messages[2].as_ref().unwrap().signature);
        assert_eq!(signature("9"), validator.tracked_messages[3].as_ref().unwrap().signature);
        assert_eq!(signature("10"), validator.tracked_messages[4].as_ref().unwrap().signature);
        assert_eq!(signature("11"), validator.tracked_messages[5].as_ref().unwrap().signature);
        assert_eq!(signature("12"), validator.tracked_messages[6].as_ref().unwrap().signature);
        assert_eq!(signature("13"), validator.tracked_messages[7].as_ref().unwrap().signature);
        assert_eq!(signature("14"), validator.tracked_messages[8].as_ref().unwrap().signature);
        assert_eq!(signature("15"), validator.tracked_messages[9].as_ref().unwrap().signature);
        assert_eq!(signature("16"), validator.tracked_messages[10].as_ref().unwrap().signature);
        assert_eq!(signature("17"), validator.tracked_messages[11].as_ref().unwrap().signature);
        assert_eq!(signature("18"), validator.tracked_messages[12].as_ref().unwrap().signature);
        assert_eq!(signature("19"), validator.tracked_messages[13].as_ref().unwrap().signature);
        assert_eq!(signature("20"), validator.tracked_messages[14].as_ref().unwrap().signature);
        assert_eq!(signature("21"), validator.tracked_messages[15].as_ref().unwrap().signature);
        assert_eq!(signature("22"), validator.tracked_messages[16].as_ref().unwrap().signature);
        assert_eq!(signature("91"), validator.tracked_messages[17].as_ref().unwrap().signature);
        assert_eq!(signature("92"), validator.tracked_messages[18].as_ref().unwrap().signature);
        assert_eq!(signature("93"), validator.tracked_messages[19].as_ref().unwrap().signature);
        assert_eq!(signature("23"), validator.tracked_messages[20].as_ref().unwrap().signature);
    }

    fn msg(offset: i32, bits: Vec<usize>) -> ChatMessage {
        let mut msg = ChatMessage {
            message: "".to_string(),
            timestamp: 0,
            salt: 0,
            signature: None,
            message_count: VarInt(offset),
            acknowledged: FixedBitSet::new(),
        };

        let mut acknowledge = FixedBitSet::new();
        for bit_index in bits {
            acknowledge.set(bit_index);
        }

        msg.acknowledged = acknowledge;

        msg
    }

    fn signature(s: &str) -> MessageSignature {
        let mut bytes: MessageSignature = MessageSignature::new([0; 256]);

        for (i, b) in s.bytes().enumerate() {
            bytes[i] = b as Byte;
        }

        bytes
    }
}
