use std::collections::{HashSet, VecDeque};

use spherix_proto::packet::clientbound::MessageSignature;

use crate::chat::UnpackedPlayerChatMessage;

pub struct MessageSignatureCache(Vec<Option<MessageSignature>>);

impl Default for MessageSignatureCache {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SIZE)
    }
}

impl MessageSignatureCache {
    const DEFAULT_SIZE: usize = 128;

    pub fn new(size: usize) -> Self {
        let mut cache = Vec::with_capacity(size);
        for _ in 0..size {
            cache.push(None);
        }

        Self(cache)
    }

    pub fn pack(&self, signature: &MessageSignature) -> i32 {
        for i in 0..self.0.len() {
            let entry = &self.0[i];
            if entry.is_none() {
                continue
            }

            if entry.as_ref().unwrap().eq(signature) {
                return i as i32;
            }
        }

        return -1
    }

    pub fn push(&mut self, packet: UnpackedPlayerChatMessage) {
        let mut deque = VecDeque::with_capacity(packet.body.last_seen_messages.0.len() + 1);

        for prev_message in packet.body.last_seen_messages.0.into_iter() {
            deque.push_back(prev_message.0)
        }

        if packet.signature.is_none() {
            panic!("Attempt to call the method without signature present")
        }

        deque.push_back(packet.signature.unwrap());
        self.do_push(deque);
    }

    fn do_push(&mut self, mut deque: VecDeque<MessageSignature>) {
        let mut set = HashSet::with_capacity(deque.len());
        for signature in deque.iter() {
            set.insert(signature.clone());
        }

        for i in 0..self.0.len() {
            if deque.is_empty() {
                break
            }

            let signature = self.0[i].clone();
            // safety: deque.is_empty() returns false
            self.0[i] = Some(deque.pop_back().unwrap());
            if signature.is_some() {
                let signature = signature.unwrap();
                if !set.contains(&signature) {
                    deque.push_front(signature);
                }
            }
        }
    }
}
