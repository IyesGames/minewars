//! Working with individual Game Update Messages

use mw_common::{game::event::*, grid::Pos, plid::PlayerId};

/// Low-Level representation of MineWars messages.
///
/// This should directly map to the binary encodings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Msg {
    Nop,
    Player {
        plid: PlayerId,
        status: MsgPlayer,
    },
    PlayerSub {
        plid: PlayerId,
        subplid: u8,
        status: MsgPlayerSub,
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
        kind: MsgStructureKind,
        pts: u16,
    },
    Construction {
        pos: Pos,
        current: u16,
        rate: u16,
    },
    RevealStructure {
        pos: Pos,
        kind: MsgStructureKind,
    },
    DigitCapture {
        pos: Pos,
        digit: u8,
        asterisk: bool,
    },
    RevealItem {
        pos: Pos,
        item: MsgItem,
    },
    TileKind {
        pos: Pos,
        kind: MsgTileKind,
    },
    TileOwner {
        pos: Pos,
        plid: PlayerId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MsgPlayer {
    // TODO
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MsgPlayerSub {
    // TODO
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

impl From<MwEv> for Msg {
    fn from(event: MwEv) -> Self {
        match event {
            MwEv::Player { plid, ev } => match ev {
                PlayerEv::Eliminated => todo!(),
                PlayerEv::Surrendered => todo!(),
                PlayerEv::Protected => todo!(),
                PlayerEv::Unprotected => todo!(),
                PlayerEv::Exploded { pos, killer } => todo!(),
                PlayerEv::Timeout { millis } => todo!(),
                PlayerEv::TimeoutFinished => todo!(),
                PlayerEv::LivesRemain { lives } => todo!(),
                PlayerEv::MatchTimeRemain { secs } => todo!(),
                PlayerEv::Debug(_) => Msg::Nop,
            }
            MwEv::PlayerSub { plid, subplid, ev } => match ev {
                PlayerSubEv::Joined { name } => todo!(),
                PlayerSubEv::NetRttInfo { millis } => todo!(),
                PlayerSubEv::Disconnected => todo!(),
                PlayerSubEv::Kicked => todo!(),
                PlayerSubEv::FriendlyChat(_) => todo!(),
                PlayerSubEv::AllChat(_) => todo!(),
                PlayerSubEv::VoteStart(_) => todo!(),
                PlayerSubEv::VoteCast(_) => todo!(),
                PlayerSubEv::VoteFail(_) => todo!(),
                PlayerSubEv::VoteSuccess(_) => todo!(),
                PlayerSubEv::Debug(_) => Msg::Nop,
            }
            MwEv::Map { pos, ev } => match ev {
                MapEv::TileKind { kind } => Msg::TileKind { pos, kind: kind.into() } ,
                MapEv::Owner { plid } => Msg::TileOwner { pos, plid },
                MapEv::Digit { digit, asterisk } => Msg::DigitCapture { pos, digit, asterisk },
                MapEv::PlaceItem { kind } => Msg::RevealItem { pos, item: kind.into() } ,
                MapEv::RevealItem { kind } => Msg::RevealItem { pos, item: kind.into() } ,
                MapEv::Flag { plid } => Msg::Flag { pos, plid },
                MapEv::Unflag => Msg::Flag { pos, plid: PlayerId::Neutral },
                MapEv::Explode => Msg::Explode { pos } ,
                MapEv::Smoke { state } => if state { Msg::Smoke { pos } } else { Msg::Unsmoke { pos } },
                MapEv::StructureReveal { kind } => Msg::RevealStructure { pos, kind: kind.into() },
                MapEv::StructureHp { hp } => Msg::StructureHp { pos, hp },
                MapEv::StructureCancel => Msg::StructureGone { pos },
                MapEv::StructureGone => Msg::StructureGone { pos },
                MapEv::StructureBuildNew { kind, pts } => Msg::BuildNew { pos, kind: kind.into(), pts },
                MapEv::StructureProgress { current, rate } => Msg::Construction { pos, current, rate },
                MapEv::Debug(_) => Msg::Nop,
            }
            MwEv::Cit { cit, ev } => match ev {
                CitEv::ResUpdate { res } => Msg::CitRes { cit, res },
                CitEv::MoneyTransaction { amount } => Msg::CitMoneyTransact { cit, amount },
                CitEv::MoneyUpdate { money } => Msg::CitMoney { cit, money },
                CitEv::IncomeUpdate { money, income } => Msg::CitIncome { cit, money, income } ,
                CitEv::TradePolicy { export, import } => Msg::CitTradeInfo { cit, export, import } ,
                CitEv::Debug(_) => Msg::Nop,
            }
            MwEv::Background(ev) => match ev {
                BackgroundEv::Tremor => Msg::Tremor,
            }
            MwEv::Nop => Msg::Nop,
            MwEv::Debug(_) => Msg::Nop,
        }
    }
}
