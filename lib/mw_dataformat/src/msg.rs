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
    CitUpdate {
        cit: u8,
        res: u16,
        money: u32,
        income: u16,
    },
    RevealStructure {
        pos: Pos,
        kind: MsgStructureKind,
    },
    StructureGone {
        pos: Pos,
    },
    StructureHp {
        pos: Pos,
        hp: u8,
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
    Tremor,
    Nop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MsgItem {
    None = 0,
    Decoy = 1,
    Mine = 2,
    Flash = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MsgStructureKind {
    Road = 0,
    Bridge = 1,
    Wall = 2,
    Tower = 3,
}
