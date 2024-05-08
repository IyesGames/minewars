use crate::prelude::*;
use crate::{plid::PlayerId, grid::Pos, game::{ItemKind, StructureKind, TileKind}};

/// All events that the game client can handle.
///
/// For simplicity, we translate events from all the different game modes into this representation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(Event))]
pub struct GameEvent {
    pub plid: PlayerId,
    pub ev: MwEv,
}

impl From<(PlayerId, MwEv)> for GameEvent {
    fn from((plid, ev): (PlayerId, MwEv)) -> Self {
        GameEvent {
            plid,
            ev,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MwEv {
    Nop,
    Debug(u8, Pos),
    Player {
        plid: PlayerId,
        subplid: Option<u8>,
        ev: PlayerEv,
    },
    Tremor,
    Smoke {
        pos: Pos,
    },
    Unsmoke {
        pos: Pos,
    },
    CitMoney {
        cit: u8,
        money: u32,
    },
    CitIncome {
        cit: u8,
        money: u32,
        income: u16,
    },
    CitMoneyTransact {
        cit: u8,
        amount: i16,
    },
    CitRes {
        cit: u8,
        res: u16,
    },
    CitTradeInfo {
        cit: u8,
        export: u8,
        import: u8,
    },
    Flag {
        plid: PlayerId,
        pos: Pos,
    },
    StructureGone {
        pos: Pos,
    },
    StructureHp {
        pos: Pos,
        hp: u8,
    },
    Explode {
        pos: Pos,
    },
    BuildNew {
        pos: Pos,
        kind: StructureKind,
        pts: u16,
    },
    Construction {
        pos: Pos,
        current: u16,
        rate: u16,
    },
    RevealStructure {
        pos: Pos,
        kind: StructureKind,
    },
    DigitCapture {
        pos: Pos,
        digit: u8,
        asterisk: bool,
    },
    RevealItem {
        pos: Pos,
        item: ItemKind,
    },
    TileKind {
        pos: Pos,
        kind: TileKind,
    },
    TileOwner {
        pos: Pos,
        plid: PlayerId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerEv {
    Joined {
        name: String,
    },
    NetRttInfo {
        duration: MwDur,
    },
    Timeout {
        duration: MwDur,
    },
    TimeoutFinished,
    Exploded {
        pos: Pos,
        killer: PlayerId,
    },
    LivesRemain {
        lives: u8,
    },
    Protected,
    Unprotected,
    Eliminated,
    Surrendered,
    Disconnected,
    Kicked,
    MatchTimeRemain {
        secs: u16,
    },
    ChatFriendly {
        text: String,
    },
    ChatAll {
        text: String,
    },
    VoteNew {
        id: u8,
        l10nkey: String,
    },
    VoteNo {
        id: u8,
    },
    VoteYes {
        id: u8,
    },
    VoteFail {
        id: u8,
    },
    VotePass {
        id: u8,
    },
}

/// The priority class to use when sending a message or stream.
///
/// When using a flexible multi-stream transport such as QUIC, a separate stream
/// can be used for each class, to improve the reliability and performance of
/// the gameplay experience.
///
/// It is also acceptable to ignore this and just send all data over a single
/// (reliable, such as TCP) stream. This will work, but will result in a degraded
/// experience.
///
/// If a stream contains messages of mixed class, the top-most class should be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MessageClass {
    /// Messages that affect the game world as observed by multiple players.
    /// Reliable, ordered, highest priority.
    PvP,
    /// Messages that are seen by multiple players, but do not affect the game world.
    /// Reliable, ordered, medium priority.
    Notification,
    /// Messages that only affect the player receiving them.
    /// Reliable, ordered, lower priority.
    Personal,
    /// Messages that are not directly involved in gameplay.
    /// Reliable, ordered, lowest priority.
    Background,
    /// Messages for real-time updates that can be dropped.
    /// Can be sent as datagrams.
    Unreliable,
}

impl MwEv {
    pub fn message_class(self) -> MessageClass {
        use MessageClass::*;
        match self {
            MwEv::Nop => Unreliable,
            MwEv::Debug(_, _) => Background,
            MwEv::Player { ev: PlayerEv::NetRttInfo { .. } , .. } => Unreliable,
            MwEv::Player { ev: PlayerEv::ChatAll { .. } , .. } => Background,
            MwEv::Player { ev: PlayerEv::ChatFriendly { .. } , .. } => Background,
            MwEv::Player { ev: PlayerEv::VoteNew { .. } , .. } => Background,
            MwEv::Player { ev: PlayerEv::VoteNo { .. } , .. } => Background,
            MwEv::Player { ev: PlayerEv::VoteYes { .. } , .. } => Background,
            MwEv::Player { ev: PlayerEv::VoteFail { .. } , .. } => Background,
            MwEv::Player { ev: PlayerEv::VotePass { .. } , .. } => Background,
            MwEv::Player { .. } => Notification,
            MwEv::Tremor => Background,
            MwEv::Smoke { .. } => PvP,
            MwEv::Unsmoke { .. } => PvP,
            MwEv::CitMoney { .. } => Unreliable,
            MwEv::CitIncome { .. } => Personal,
            MwEv::CitMoneyTransact { .. } => Personal,
            MwEv::CitRes { .. } => Personal,
            MwEv::CitTradeInfo { .. } => Personal,
            MwEv::Flag { .. } => PvP,
            MwEv::StructureGone { .. } => PvP,
            MwEv::StructureHp { .. } => PvP,
            MwEv::Explode { .. } => PvP,
            MwEv::BuildNew { .. } => Personal,
            MwEv::Construction { .. } => Unreliable,
            MwEv::RevealStructure { .. } => PvP,
            MwEv::DigitCapture { .. } => PvP,
            MwEv::RevealItem { .. } => PvP,
            MwEv::TileKind { .. } => PvP,
            MwEv::TileOwner { .. } => PvP,
        }
    }
}

