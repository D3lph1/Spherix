// PlayerInfoUpdate packet has substandard format, so it is required to write
// custom implementation of Writeable and Readable traits for them.

use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};

use anyhow::anyhow;
use uuid::Uuid;

use crate::io::{Byte, Error, Long, VarInt};
use crate::io::{Readable, Writable};
use crate::packet::{packet, packet_wo_io};

packet!(
    PlayerInfoUpdateActionAddPlayer {
        name: String,
        properties: Vec<PlayerInfoUpdateActionAddPlayerProperty>
    }

    PlayerInfoUpdateActionAddPlayerProperty {
        name: String,
        value: String,
        signature: Option<String>
    }

    PlayerInfoUpdateActionInitializeChat {
        signature: Option<PlayerInfoUpdateActionInitializeChatSignature>
    }

    PlayerInfoUpdateActionInitializeChatSignature {
        chat_session_id: Uuid,
        public_key_expiry_time: Long,
        encoded_public_key: Box<[Byte]>,
        public_key_signature: Box<[Byte]>
    }

    PlayerInfoUpdateActionUpdateGameMode {
        game_mode: VarInt
    }

    PlayerInfoUpdateActionUpdateListed {
        listed: bool
    }

    PlayerInfoUpdateActionUpdateLatency {
        ping: VarInt
    }

    PlayerInfoUpdateActionUpdateDisplayName {
        display_name: Option<String>
    }
);

packet_wo_io!(
    PlayerInfoUpdate {
        actions_bitmap: Byte,
        action_sets: Vec<PlayerInfoUpdateActionSet>
    }
);

impl PlayerInfoUpdate {
    const MASK: Byte = 0b00111111;
    const INVALID_MASK: Byte = !Self::MASK;

    pub fn try_new(actions_bitmap: Byte) -> Result<Self, anyhow::Error> {
        if actions_bitmap & Self::INVALID_MASK != 0 {
            return Err(anyhow!("Invalid bitmap"));
        }

        Ok(Self {
            actions_bitmap,
            action_sets: vec![],
        })
    }

    pub fn push_action_set(&mut self, action_set: PlayerInfoUpdateActionSet) -> Result<(), anyhow::Error> {
        for (action, _) in action_set.actions.iter() {
            if action.bitmask() & Self::MASK == 0 {
                return Err(anyhow!("bitset does not contain bit for this type of action"));
            }
        }

        self.action_sets.push(action_set);

        Ok(())
    }
}

impl Writable for PlayerInfoUpdate {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        let mut written = self.actions_bitmap.write(buf)?;

        written += VarInt(self.action_sets.len() as i32).write(buf)?;
        for action_set in self.action_sets.iter() {
            written += action_set.uuid.write(buf)?;

            let actions = PlayerInfoUpdateAction::order_hashmap(&action_set.actions);
            for action in actions {
                written += action.write(buf)?;
            }
        }

        Ok(written)
    }
}

impl Readable for PlayerInfoUpdate {
    fn read<R: Read>(buf: &mut R) -> Result<Self, Error> where Self: Sized {
        let bitmap = Byte::read(buf)?;

        let action_sets_len = VarInt::read(buf)?.0;
        let mut action_sets = Vec::with_capacity(action_sets_len as usize);
        for _ in 0..action_sets_len {
            let action_set = PlayerInfoUpdateActionSet::read_by_bitmap(buf, bitmap)?;
            action_sets.push(action_set);
        }

        Ok(Self {
            actions_bitmap: bitmap,
            action_sets,
        })
    }
}

packet_wo_io!(
    PlayerInfoUpdateActionSet {
        uuid: Uuid,
        actions: HashMap<PlayerInfoUpdateAction, ()>
    }
);

impl PlayerInfoUpdateActionSet {
    fn read_by_bitmap<R: Read>(buf: &mut R, bitmap: Byte) -> Result<Self, Error> where Self: Sized {
        let uuid = Uuid::read(buf)?;

        let mut actions = HashMap::new();

        if bitmap & 0b00000001 != 0 {
            actions.insert(PlayerInfoUpdateAction::AddPlayer(PlayerInfoUpdateActionAddPlayer::read(buf)?), ());
        }

        if bitmap & 0b00000010 != 0 {
            actions.insert(PlayerInfoUpdateAction::InitializeChat(PlayerInfoUpdateActionInitializeChat::read(buf)?), ());
        }

        if bitmap & 0b00000100 != 0 {
            actions.insert(PlayerInfoUpdateAction::UpdateGameMode(PlayerInfoUpdateActionUpdateGameMode::read(buf)?), ());
        }

        if bitmap & 0b00001000 != 0 {
            actions.insert(PlayerInfoUpdateAction::UpdateListed(PlayerInfoUpdateActionUpdateListed::read(buf)?), ());
        }

        if bitmap & 0b00010000 != 0 {
            actions.insert(PlayerInfoUpdateAction::UpdateLatency(PlayerInfoUpdateActionUpdateLatency::read(buf)?), ());
        }

        if bitmap & 0b00100000 != 0 {
            actions.insert(PlayerInfoUpdateAction::UpdateDisplayName(PlayerInfoUpdateActionUpdateDisplayName::read(buf)?), ());
        }

        Ok(Self {
            uuid,
            actions
        })
    }
}

#[derive(Debug, Clone)]
pub enum PlayerInfoUpdateAction {
    AddPlayer(PlayerInfoUpdateActionAddPlayer),
    InitializeChat(PlayerInfoUpdateActionInitializeChat),
    UpdateGameMode(PlayerInfoUpdateActionUpdateGameMode),
    UpdateListed(PlayerInfoUpdateActionUpdateListed),
    UpdateLatency(PlayerInfoUpdateActionUpdateLatency),
    UpdateDisplayName(PlayerInfoUpdateActionUpdateDisplayName),
}

/// We don't care about internal structs equality because each action set
/// contains only actions of unique types.
impl PartialEq for PlayerInfoUpdateAction {
    fn eq(&self, other: &Self) -> bool {
        self.bitmask() == other.bitmask()
    }
}

impl Eq for PlayerInfoUpdateAction {}

impl Hash for PlayerInfoUpdateAction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_i8(self.bitmask());
    }
}

impl PlayerInfoUpdateAction {
    pub const BITMASK_ADD_PLAYER: Byte = 0b00000001;
    pub const BITMASK_INITIALIZE_CHAT: Byte = 0b00000010;
    pub const BITMASK_UPDATE_GAME_MODE: Byte = 0b00000100;
    pub const BITMASK_UPDATE_LISTED: Byte = 0b00001000;
    pub const BITMASK_UPDATE_LATENCY: Byte = 0b00010000;
    pub const BITMASK_UPDATE_DISPLAY_MODE: Byte = 0b00100000;

    fn bitmask(&self) -> Byte {
        match self {
            PlayerInfoUpdateAction::AddPlayer(_) => Self::BITMASK_ADD_PLAYER,
            PlayerInfoUpdateAction::InitializeChat(_) => Self::BITMASK_INITIALIZE_CHAT,
            PlayerInfoUpdateAction::UpdateGameMode(_) => Self::BITMASK_UPDATE_GAME_MODE,
            PlayerInfoUpdateAction::UpdateListed(_) => Self::BITMASK_UPDATE_LISTED,
            PlayerInfoUpdateAction::UpdateLatency(_) => Self::BITMASK_UPDATE_LATENCY,
            PlayerInfoUpdateAction::UpdateDisplayName(_) => Self::BITMASK_UPDATE_DISPLAY_MODE,
        }
    }

    pub fn order_hashmap(map: &HashMap<Self, ()>) -> Vec<&Self> {
        let mut vec: Vec<_> = map.keys().collect();
        vec.sort_by(|a, b| a.bitmask().cmp(&b.bitmask()));

        vec
    }

    pub fn bitmask_or(these: &Vec<Self>) -> Byte {
        let mut bitmask = 0;

        for this in these {
            bitmask |= this.bitmask();
        }

        bitmask
    }
}

impl Writable for PlayerInfoUpdateAction {
    fn write<W: Write>(&self, buf: &mut W) -> Result<usize, Error> {
        match self {
            PlayerInfoUpdateAction::AddPlayer(x) => x.write(buf),
            PlayerInfoUpdateAction::InitializeChat(x) => x.write(buf),
            PlayerInfoUpdateAction::UpdateGameMode(x) => x.write(buf),
            PlayerInfoUpdateAction::UpdateListed(x) => x.write(buf),
            PlayerInfoUpdateAction::UpdateLatency(x) => x.write(buf),
            PlayerInfoUpdateAction::UpdateDisplayName(x) => x.write(buf),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::io::Cursor;

    use uuid::Uuid;

    use crate::io::VarInt;
    use crate::io::{Readable, Writable};

    use super::{PlayerInfoUpdate, PlayerInfoUpdateAction, PlayerInfoUpdateActionAddPlayer, PlayerInfoUpdateActionInitializeChat, PlayerInfoUpdateActionSet, PlayerInfoUpdateActionUpdateLatency};

    #[test]
    fn test() {
        let mut buf = Vec::new();

        let action_sets = vec![
            PlayerInfoUpdateAction::AddPlayer(PlayerInfoUpdateActionAddPlayer {
                name: "D3lph1".to_owned(),
                properties: vec![],
            }),
            PlayerInfoUpdateAction::InitializeChat(PlayerInfoUpdateActionInitializeChat {
                signature: None,
            }),
            PlayerInfoUpdateAction::UpdateLatency(PlayerInfoUpdateActionUpdateLatency {
                ping: VarInt(71),
            })
        ];

        let bitmask = PlayerInfoUpdateAction::bitmask_or(&action_sets);

        let mut written_packet = PlayerInfoUpdate::try_new(bitmask).unwrap();
        let mut actions = HashMap::new();
        for action_set in action_sets {
            actions.insert(action_set, ());
        }

        written_packet.push_action_set(PlayerInfoUpdateActionSet {
            uuid: Uuid::new_v4(),
            actions
        }).unwrap();

        written_packet.write(&mut buf).unwrap();

        let mut cursor = Cursor::new(buf);

        let read_packet = PlayerInfoUpdate::read(&mut cursor).unwrap();

        assert_eq!(written_packet.actions_bitmap, read_packet.actions_bitmap);

        for (i, read_action_set) in read_packet.action_sets.into_iter().enumerate() {
            let written_action_set = &written_packet.action_sets[i];
            assert_eq!(written_action_set.uuid, read_action_set.uuid);
            assert_eq!(written_action_set.actions.len(), read_action_set.actions.len());

            for (read_action, _) in read_action_set.actions.iter() {
                assert!(written_action_set.actions.contains_key(read_action));
            }
        }
    }
}
