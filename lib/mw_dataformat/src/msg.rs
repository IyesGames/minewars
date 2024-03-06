//! Working with individual Game Update Messages

use mw_common::{grid::Pos, plid::PlayerId};

/// Logical representation of a protocol message.
///
/// This is the IR used as a final step before encoding raw bytes.
///
/// The sorting order is important! Optimization passes rely on it! Do not reorder things in this enum!
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Msg {
    Player {
        plid: PlayerId,
        status: u8,
    },
    Capture {
        pos: Pos,
        digit: u8,
    },
    TileOwner {
        pos: Pos,
        plid: PlayerId,
    },
    Digit {
        pos: Pos,
        digit: u8,
    },
    CitRes {
        cit: u8,
        res: u16,
    },
    CitSpend {
        cit: u8,
        spent: u16,
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
    CitTradeInfo {
        cit: u8,
        export: u8,
        import: u8,
    },
    RevealStructure {
        pos: Pos,
        kind: MsgStructureKind,
    },
    StructureHp {
        pos: Pos,
        hp: u8,
    },
    StructureCancel {
        pos: Pos,
    },
    StructureGone {
        pos: Pos,
    },
    BuildNew {
        pos: Pos,
        kind: MsgStructureKind,
        pts: u16,
    },
    Construction {
        pos: Pos,
        current: u16,
        rate: u16,
    },
    PlaceItem {
        pos: Pos,
        item: MsgItem,
    },
    RevealItem {
        pos: Pos,
        item: MsgItem,
    },
    Explode {
        pos: Pos,
    },
    Smoke {
        pos: Pos,
    },
    Unsmoke {
        pos: Pos,
    },
    Tremor,
    Nop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MsgTileKind {
    Water = 0,
    Mountain = 2,
    Forest = 3,
    Destroyed = 4,
    Foundation = 5,
    Regular = 6,
    Fertile = 7,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MsgItem {
    None = 0,
    Decoy = 1,
    Mine = 2,
    Trap = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MsgStructureKind {
    Road = 0,
    Bridge = 1,
    Wall = 2,
    Tower = 3,
}

impl From<mw_common::game::TileKind> for MsgTileKind {
    fn from(value: mw_common::game::TileKind) -> Self {
        match value {
            mw_common::game::TileKind::Water => MsgTileKind::Water,
            mw_common::game::TileKind::FoundationRoad => MsgTileKind::Foundation,
            mw_common::game::TileKind::FoundationStruct => MsgTileKind::Foundation,
            mw_common::game::TileKind::Regular => MsgTileKind::Regular,
            mw_common::game::TileKind::Fertile => MsgTileKind::Fertile,
            mw_common::game::TileKind::Forest => MsgTileKind::Forest,
            mw_common::game::TileKind::Mountain => MsgTileKind::Mountain,
            mw_common::game::TileKind::Destroyed => MsgTileKind::Destroyed,
        }
    }
}

impl From<mw_common::game::ItemKind> for MsgItem {
    fn from(value: mw_common::game::ItemKind) -> Self {
        match value {
            mw_common::game::ItemKind::Safe => MsgItem::None,
            mw_common::game::ItemKind::Mine => MsgItem::Mine,
            mw_common::game::ItemKind::Decoy => MsgItem::Decoy,
            mw_common::game::ItemKind::Trap => MsgItem::Trap,
        }
    }
}

impl From<mw_common::game::StructureKind> for MsgStructureKind {
    fn from(value: mw_common::game::StructureKind) -> Self {
        match value {
            mw_common::game::StructureKind::Road => MsgStructureKind::Road,
            mw_common::game::StructureKind::Barricade => MsgStructureKind::Wall,
            mw_common::game::StructureKind::WatchTower => MsgStructureKind::Tower,
            mw_common::game::StructureKind::Bridge => MsgStructureKind::Bridge,
        }
    }
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

impl Msg {
    pub fn message_class(self) -> MessageClass {
        match self {
            Msg::Player { .. } => MessageClass::Notification,
            Msg::Capture { .. } => MessageClass::PvP,
            Msg::TileOwner { .. } => MessageClass::PvP,
            Msg::Digit { .. } => MessageClass::PvP,
            Msg::CitRes { .. } => MessageClass::Personal,
            Msg::CitSpend { .. } => MessageClass::Personal,
            Msg::CitMoney { .. } => MessageClass::Unreliable,
            Msg::CitIncome { .. } => MessageClass::Personal,
            Msg::CitTradeInfo { .. } => MessageClass::Personal,
            Msg::RevealStructure { .. } => MessageClass::PvP,
            Msg::StructureGone { .. } => MessageClass::PvP,
            Msg::StructureCancel { .. } => MessageClass::Personal,
            Msg::StructureHp { .. } => MessageClass::PvP,
            Msg::BuildNew { .. } => MessageClass::Personal,
            Msg::Construction { .. } => MessageClass::Unreliable,
            Msg::RevealItem { .. } => MessageClass::PvP,
            Msg::PlaceItem { .. } => MessageClass::Personal,
            Msg::Explode { .. } => MessageClass::PvP,
            Msg::Smoke { .. } => MessageClass::PvP,
            Msg::Unsmoke { .. } => MessageClass::PvP,
            Msg::Tremor => MessageClass::Background,
            Msg::Nop => MessageClass::Unreliable,
        }
    }
}
