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
    PlayerSub {
        plid: PlayerId,
        subplid: u8,
        ev: PlayerSubEv,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlayerSubEv {
    Joined {
        name: String,
    },
    NetRttInfo {
        millis: u8,
    },
    Disconnected,
    Kicked,
    FriendlyChat(String),
    AllChat(String),
    VoteStart(String),
    VoteCast(String),
    VoteFail(String),
    VoteSuccess(String),
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CitEv {
    ResUpdate {
        res: u16,
    },
    MoneyTranaction {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackgroundEv {
    Tremor,
}

