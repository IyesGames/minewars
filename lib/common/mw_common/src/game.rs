use enum_iterator::Sequence;
use enum_map::{Enum, EnumMap};
use num_derive::FromPrimitive;
use modular_bitfield::prelude::*;

use crate::prelude::*;
use crate::grid::*;
use crate::plid::*;

pub type CitId = u8;

/// The possibilities of what can be on a given tile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(Enum, FromPrimitive, ToPrimitive, BitfieldSpecifier, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
#[bits = 2]
pub enum ItemKind {
    #[default]
    Safe,
    Mine,
    Decoy,
    Trap,
}

/// The base variant of a map tile
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(Enum, FromPrimitive, ToPrimitive, BitfieldSpecifier, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
#[bits = 3]
pub enum TileKind {
    Water,
    Destroyed,
    #[default]
    Regular,
    Fertile,
    FoundationStruct,
    FoundationRoad,
    Forest,
    Mountain,
}

impl TileKind {
    pub const fn ownable(self) -> bool {
        match self {
            TileKind::Water => false,
            _ => true,
        }
    }

    pub const fn is_land(self) -> bool {
        match self {
            TileKind::Water |
            TileKind::Forest |
            TileKind::Mountain |
            TileKind::FoundationStruct => false,
            TileKind::Regular |
            TileKind::Fertile |
            TileKind::Destroyed |
            TileKind::FoundationRoad => true,
        }
    }

    pub const fn is_rescluster(self) -> bool {
        match self {
            TileKind::Forest |
            TileKind::Mountain => true,
            TileKind::Water |
            TileKind::Regular |
            TileKind::Fertile |
            TileKind::Destroyed |
            TileKind::FoundationRoad |
            TileKind::FoundationStruct => false,
        }
    }

    pub const fn is_harvestable(self) -> bool {
        match self {
            TileKind::Water |
            TileKind::Destroyed |
            TileKind::FoundationRoad |
            TileKind::FoundationStruct => false,
            TileKind::Regular |
            TileKind::Fertile |
            TileKind::Forest |
            TileKind::Mountain => true,
        }
    }
}

/// Is a structure in-production or completed?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
pub enum ProdState {
    Pending,
    Built,
}

/// All the various structures that can be built
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[derive(Enum, FromPrimitive, ToPrimitive, BitfieldSpecifier, Sequence)]
#[cfg_attr(feature = "bevy", derive(Component))]
#[bits = 2]
pub enum StructureKind {
    #[default]
    Road,
    Barricade,
    WatchTower,
    Bridge,
}

/// The gameplay actions that a player can perform on a tile
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[derive(Enum, Sequence)]
pub enum ActionKind {
    // PvP "offense"
    Explore,
    Strike,
    Reveal,
    // PvP "defense"
    Deploy,
    Undeploy,
    Smoke,
    // Base Building
    Harvest,
    Build,
    Bulldoze,
}

/// Rules (Balancing Table)
///
/// This struct contains all values that the client needs
/// to be aware of, in order to display accurate info about
/// the game state to the user.
///
/// The server uses this + more (additional rules for the
/// MineWars Game) to actually run the game.
#[cfg_attr(feature = "bevy", derive(Component))]
pub struct MwRules {
    pub res_base: u8,
    pub res_tile: EnumMap<TileKind, u8>,
    pub res_tile_harvest: EnumMap<TileKind, u16>,
    pub cit_starting_money_spawn: u16,
    pub cit_starting_money_other: u16,
    pub mult_cost_foreign_region: MwRatio,
    pub mult_costsharing_local_contribution: MwRatio,
    pub vis_radius: u8,
    pub vis_radius_watchtower: u8,
    pub structure_return_bulldoze: EnumMap<StructureKind, u16>,
    pub structure_return_cancel: EnumMap<StructureKind, u16>,
    pub structure_cost_initial: EnumMap<StructureKind, u16>,
    pub structure_cost_construction: EnumMap<StructureKind, u16>,
    pub action_cost: EnumMap<ActionKind, u16>,
    pub action_cooldown: EnumMap<ActionKind, MwDur>,
    pub action_delay: EnumMap<ActionKind, MwDur>,
    pub item_cost: EnumMap<ItemKind, u16>,
    pub item_return: EnumMap<ItemKind, u16>,
    pub cost_strike_tile: u16,
    pub dur_land_protect: MwDur,
    pub dur_capture_city: MwDur,
}

/// The basic per-tile data, as per mapgen or file
#[bitfield]
#[derive(Clone, Copy, Default)]
pub struct MapGenTileData {
    pub kind: TileKind,
    pub mark: bool,
    pub item: ItemKind,
    #[skip] __: B2,
    pub region: CitId,
}

/// A MineWars Digit Value
///
/// This is an "enhanced" Minesweeper digit.
///
/// The `u8` is the digit value (`0` means no digit).
/// The `bool` is whether to display an asterisk.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MwDigit {
    pub digit: u8,
    pub asterisk: bool,
}

/// All events that the game client can handle.
///
/// For simplicity, we translate events from all the different game modes into this representation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(Event))]
pub struct GameEvent {
    pub plids: Plids,
    pub ev: MwEv,
}

impl From<(Plids, MwEv)> for GameEvent {
    fn from((plids, ev): (Plids, MwEv)) -> Self {
        GameEvent {
            plids,
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
        cit: CitId,
        money: u32,
    },
    CitIncome {
        cit: CitId,
        money: u32,
        income: u16,
    },
    CitMoneyTransact {
        cit: CitId,
        amount: i16,
    },
    CitRes {
        cit: CitId,
        res: u16,
    },
    CitTradeInfo {
        cit: CitId,
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
        digit: MwDigit,
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
