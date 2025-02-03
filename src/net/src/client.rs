use std::fmt::{Debug, Formatter};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Mutex, MutexGuard};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use flume::{Receiver, Sender};
use uuid::Uuid;

use spherix_proto::packet::clientbound::{KeepAlive as KeepAlivePacket, PlayMapping as ClientboundPlayMapping};
use spherix_proto::packet::serverbound::{ChatMessage, PlayMapping as ServerboundPlayMapping};

use crate::chat::{LastSeenMessagesValidator, MessageSignatureCache, UnpackedLastSeenMessages, UnpackedPlayerChatMessage, UnpackedSignedMessageBody};

pub struct Client {
    pub name: String,
    pub uuid: Uuid,
    pub received: Receiver<ServerboundPlayMapping>,
    pub to_send: Sender<ClientboundPlayMapping>,
    pub keep_alive: Mutex<KeepAlive>,
    pub session: Mutex<Option<Session>>,
    pub last_chat_timestamp: AtomicU64,
    pub last_seen_messages: Mutex<LastSeenMessagesValidator>,
    pub message_signature_cache: Mutex<MessageSignatureCache>,
    pub message_index: AtomicUsize
}

impl Client {
    pub fn new(
        name: String,
        uuid: Uuid,
        received: Receiver<ServerboundPlayMapping>,
        to_send: Sender<ClientboundPlayMapping>,
    ) -> Self {
        Self {
            name,
            uuid,
            received,
            to_send: to_send.clone(),
            keep_alive: Mutex::new(KeepAlive::new(to_send)),
            session: Mutex::new(None),
            last_chat_timestamp: AtomicU64::new(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()),
            last_seen_messages: Mutex::new(LastSeenMessagesValidator::new(20)),
            message_signature_cache: Mutex::new(MessageSignatureCache::default()),
            message_index: AtomicUsize::new(0)
        }
    }

    pub fn send_packet(&self, packet: ClientboundPlayMapping) {
        self.to_send.try_send(packet).unwrap();
    }

    pub fn keep_alive(&self) -> MutexGuard<'_, KeepAlive> {
        return self.keep_alive.lock().unwrap()
    }

    pub fn session(&self) -> MutexGuard<'_, Option<Session>> {
        self.session.lock().unwrap()
    }

    pub fn set_session(&self, session: Session) {
        *self.session.lock().unwrap() = Some(session)
    }

    pub fn clear_session(&self) {
        *self.session.lock().unwrap() = None
    }

    pub fn handle_chat(&self, chat_message: ChatMessage) -> Option<UnpackedPlayerChatMessage> {
        let optional = self.try_handle_chat(chat_message.clone())?;

        Some(self.get_signed_message(chat_message, optional))
    }

    pub fn try_handle_chat(&self, chat_message: ChatMessage) -> Option<UnpackedLastSeenMessages> {
        self.unpack_and_apply_last_seen(chat_message)
    }

    fn unpack_and_apply_last_seen(&self, chat_message: ChatMessage) -> Option<UnpackedLastSeenMessages> {
        self.last_seen_messages.lock().unwrap().apply_update(chat_message)
    }

    fn get_signed_message(&self, chat_message: ChatMessage, last_seen: UnpackedLastSeenMessages) -> UnpackedPlayerChatMessage {
        let signed_body = UnpackedSignedMessageBody {
            content: chat_message.message,
            salt: chat_message.salt,
            timestamp: chat_message.timestamp,
            last_seen_messages: last_seen,
        };

        return UnpackedPlayerChatMessage {
            signature: chat_message.signature,
            body: signed_body,
        }
    }

    pub fn add_pending_message(&self, p: UnpackedPlayerChatMessage) {
        if p.signature.is_none() {
            return;
        }

        let signature = p.signature.clone().unwrap();
        self.message_signature_cache.lock().unwrap().push(p);

        let mut last_seen_messages = self.last_seen_messages.lock().unwrap();
        last_seen_messages.add_pending(signature);
    }

    pub fn message_index(&self) -> usize {
        self.message_index.load(Ordering::Acquire)
    }

    pub fn inc_message_index(&self) {
        self.message_index.fetch_add(1, Ordering::Release);
    }
}

impl Debug for Client {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Client {{name: {}, uuid: {}}}", self.name, self.uuid)
    }
}

pub struct KeepAlive {
    id: i64,
    last_at: Instant,
    to_send: Sender<ClientboundPlayMapping>
}

impl KeepAlive {
    fn new(to_send: Sender<ClientboundPlayMapping>) -> Self {
        Self {
            id: Self::next_id(),
            last_at: Instant::now(),
            to_send
        }
    }

    #[inline]
    pub fn is_time_to(&self, freq: Duration) -> bool {
        Instant::now() - self.last_at >= freq
    }

    #[inline]
    pub fn send(&mut self) {
        self.to_send.try_send(ClientboundPlayMapping::KeepAlive(KeepAlivePacket {
            keep_alive_id: self.id
        })).unwrap();

        self.last_at = Instant::now();
    }

    #[inline]
    pub fn set_next_id(&mut self) {
        self.id = Self::next_id();
    }

    #[inline]
    pub fn check(&self, incoming_id: i64) -> bool {
        self.id == incoming_id
    }

    #[inline]
    fn next_id() -> i64 {
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
    }
}

pub struct Session {
    pub session_id: Uuid,
    pub expires_at: SystemTime,
    pub public_key: Box<[i8]>,
    pub key_signature: Box<[i8]>
}

impl Session {
    pub fn new(
        session_id: Uuid,
        expires_at: SystemTime,
        public_key: Box<[i8]>,
        key_signature: Box<[i8]>
    ) -> Self {
        Self {
            session_id,
            expires_at,
            public_key,
            key_signature
        }
    }
}
