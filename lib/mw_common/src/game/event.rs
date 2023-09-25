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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerEv {
    Joined,
    Disconnected,
    Eliminated,
    Surrendered,
    Protected,
    Unprotected,
    Kicked,
    Exploded {
        pos: Pos,
    },
    Timeout {
        millis: u16,
    },
    TimeoutFinished,
    Flash {
        millis: u16,
    },
    FlashFinished,
    LivesRemain {
        lives: u8,
    },
    FriendlyChat(String),
    AllChat(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapEv {
    Tile {
        kind: TileKind,
    },
    Owner {
        plid: PlayerId,
    },
    Digit {
        digit: u8,
        asterisk: bool,
    },
    Item {
        kind: ItemKind,
    },
    Flag {
        plid: PlayerId,
    },
    Explode,
    Smoke {
        state: bool,
    },
    StructureBegin {
        kind: StructureKind,
        pts: u16,
    },
    StructureReveal {
        kind: StructureKind,
    },
    StructureHp {
        hp: u8,
    },
    StructureProgress {
        current: u16,
        rate: u16,
    },
    StructureGone,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitEv {
    Money {
        current: u32,
        income: u16,
    },
    Spent {
        amount: u16,
    },
    ResAvailable {
        res: u16,
    },
    TradePolicy {
        import: u8,
        export: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundEv {
    Tremor,
}

