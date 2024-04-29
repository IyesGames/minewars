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
    Player {
        plid: PlayerId,
        subplid: Option<u8>,
        ev: PlayerEv,
    },
    Map {
        pos: Pos,
        ev: MapEv,
    },
    Cit {
        cit: u8,
        ev: CitEv,
    },
    Background(BackgroundEv),
    Nop,
    Debug(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerEv {
    Eliminated,
    Surrendered,
    Protected,
    Unprotected,
    Exploded {
        pos: Pos,
        killer: PlayerId,
    },
    Timeout {
        millis: u16,
    },
    TimeoutFinished,
    LivesRemain {
        lives: u8,
    },
    MatchTimeRemain {
        secs: u16,
    },
    Joined,
    NetRttInfo {
        millis: u16,
    },
    Disconnected,
    Kicked,
    FriendlyChat(String),
    AllChat(String),
    VoteStart(String),
    VoteCast(String),
    VoteFail(String),
    VoteSuccess(String),
    Debug(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapEv {
    TileKind {
        kind: TileKind,
    },
    Owner {
        plid: PlayerId,
    },
    Digit {
        digit: u8,
        asterisk: bool,
    },
    PlaceItem {
        kind: ItemKind,
    },
    RevealItem {
        kind: ItemKind,
    },
    Flag {
        plid: PlayerId,
    },
    Unflag,
    Explode,
    Smoke {
        state: bool,
    },
    StructureReveal {
        kind: StructureKind,
    },
    StructureHp {
        hp: u8,
    },
    StructureCancel,
    StructureGone,
    StructureBuildNew {
        kind: StructureKind,
        pts: u16,
    },
    StructureProgress {
        current: u16,
        rate: u16,
    },
    Debug(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitEv {
    ResUpdate {
        res: u16,
    },
    MoneyTransaction {
        amount: i16,
    },
    MoneyUpdate {
        money: u32,
    },
    IncomeUpdate {
        money: u32,
        income: u16,
    },
    TradePolicy {
        export: u8,
        import: u8,
    },
    Debug(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundEv {
    Tremor,
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
    /// Reliable, unordered, lowest priority.
    Background,
    /// Messages for real-time updates that can be dropped.
    /// Can be sent as datagrams.
    Unreliable,
}

impl MwEv {
    pub fn message_class(self) -> MessageClass {
        match self {
            MwEv::Nop => MessageClass::Unreliable,
            MwEv::Debug(_) => MessageClass::Unreliable,
            MwEv::Player { ev: PlayerEv::Debug(_), .. } => MessageClass::Unreliable,
            MwEv::Player { ev: PlayerEv::NetRttInfo { .. }, .. } => MessageClass::Unreliable,
            MwEv::Player { .. } => MessageClass::Notification,
            MwEv::Background(_)  => MessageClass::Background,
            MwEv::Cit { ev, .. } => match ev {
                CitEv::ResUpdate { .. } => MessageClass::Personal,
                CitEv::MoneyTransaction { .. } => MessageClass::Personal,
                CitEv::MoneyUpdate { .. } => MessageClass::Unreliable,
                CitEv::IncomeUpdate { .. } => MessageClass::Personal,
                CitEv::TradePolicy { .. } => MessageClass::Personal,
                CitEv::Debug(_) => MessageClass::Unreliable,
            },
            MwEv::Map { ev, .. } => match ev {
                MapEv::TileKind { .. } => MessageClass::PvP,
                MapEv::Owner { .. } => MessageClass::PvP,
                MapEv::Digit { .. } => MessageClass::PvP,
                MapEv::PlaceItem { .. } => MessageClass::Personal,
                MapEv::RevealItem { .. } => MessageClass::PvP,
                MapEv::Flag { .. } => MessageClass::PvP,
                MapEv::Unflag => MessageClass::PvP,
                MapEv::Explode => MessageClass::PvP,
                MapEv::Smoke { .. } => MessageClass::PvP,
                MapEv::StructureReveal { .. } => MessageClass::PvP,
                MapEv::StructureHp { .. } => MessageClass::PvP,
                MapEv::StructureCancel => MessageClass::Personal,
                MapEv::StructureGone => MessageClass::PvP,
                MapEv::StructureBuildNew { .. } => MessageClass::Personal,
                MapEv::StructureProgress { .. } => MessageClass::Unreliable,
                MapEv::Debug(_) => MessageClass::Unreliable,
            },
        }
    }
}

