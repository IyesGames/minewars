//! Working with individual Game Update Messages

use mw_common::{game::event::*, grid::Pos, plid::PlayerId};
use num_derive::*;

use crate::time::MwDur;

pub mod asm;
pub mod bin;

pub trait MsgWriter {
    type Error: std::error::Error;
    fn write<W: std::io::Write>(&mut self, w: &mut W, msgs: &[Msg], max_bytes: usize) -> Result<(usize, usize), Self::Error>;
    fn write_many<W: std::io::Write>(&mut self, w: &mut W, mut msgs: &[Msg], max_bytes: usize) -> Result<(usize, usize), Self::Error> {
        let mut msgs_total = 0;
        let mut bytes_total = 0;
        while !msgs.is_empty() {
            let (msgs_written, bytes_written) = self.write(w, msgs, max_bytes - bytes_total)?;
            if msgs_written == 0 {
                break;
            }
            msgs_total += msgs_written;
            bytes_total += bytes_written;
            msgs = &msgs[msgs_written..];
        }
        Ok((msgs_total, bytes_total))
    }
    fn write_all<W: std::io::Write>(&mut self, w: &mut W, mut msgs: &[Msg]) -> Result<(), Self::Error> {
        while !msgs.is_empty() {
            let (msgs_written, _) = self.write(w, msgs, usize::MAX)?;
            if msgs_written == 0 {
                break;
            }
            msgs = &msgs[msgs_written..];
        }
        Ok(())
    }
}

pub trait MsgReader {
    type Error: std::error::Error;
    fn read<R: std::io::BufRead>(&mut self, r: &mut R, out: &mut Vec<Msg>) -> Result<usize, Self::Error>;
    fn read_all<R: std::io::BufRead>(&mut self, r: &mut R, out: &mut Vec<Msg>) -> Result<usize, Self::Error> {
        let mut total = 0;
        loop {
            let len_prev = out.len();
            total += self.read(r, out)?;
            if len_prev == out.len() {
                break;
            }
        }
        Ok(total)
    }
}

/// Low-Level representation of MineWars messages.
///
/// This should directly map to the binary encodings.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Msg {
    Nop,
    Player {
        plid: PlayerId,
        subplid: Option<u8>,
        status: MsgPlayer,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MsgPlayer {
    Joined,
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(FromPrimitive, ToPrimitive)]
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
#[derive(FromPrimitive, ToPrimitive)]
pub enum MsgItem {
    None = 0,
    Decoy = 1,
    Mine = 2,
    Trap = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(FromPrimitive, ToPrimitive)]
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
            MwEv::Player { plid, subplid, ev } => match ev {
                PlayerEv::Eliminated => Msg::Player { plid, subplid, status: MsgPlayer::Eliminated },
                PlayerEv::Surrendered => Msg::Player { plid, subplid, status: MsgPlayer::Surrendered },
                PlayerEv::Protected => Msg::Player { plid, subplid, status: MsgPlayer::Protected },
                PlayerEv::Unprotected => Msg::Player { plid, subplid, status: MsgPlayer::Unprotected },
                PlayerEv::Exploded { pos, killer } => Msg::Player { plid, subplid, status: MsgPlayer::Exploded { pos, killer } },
                PlayerEv::Timeout { millis } => Msg::Player { plid, subplid, status: MsgPlayer::Timeout { duration: MwDur::from_millis_lossy(millis) } },
                PlayerEv::TimeoutFinished => Msg::Player { plid, subplid, status: MsgPlayer::TimeoutFinished },
                PlayerEv::LivesRemain { lives } => Msg::Player { plid, subplid, status: MsgPlayer::LivesRemain { lives } },
                PlayerEv::MatchTimeRemain { secs } => Msg::Player { plid, subplid, status: MsgPlayer::MatchTimeRemain { secs } },
                PlayerEv::Joined => Msg::Player { plid, subplid, status: MsgPlayer::Joined },
                PlayerEv::NetRttInfo { millis } => Msg::Player { plid, subplid, status: MsgPlayer::NetRttInfo { duration: MwDur::from_millis_lossy(millis) } },
                PlayerEv::Disconnected => Msg::Player { plid, subplid, status: MsgPlayer::Disconnected },
                PlayerEv::Kicked => Msg::Player { plid, subplid, status: MsgPlayer::Kicked },
                PlayerEv::FriendlyChat(_) => todo!(),
                PlayerEv::AllChat(_) => todo!(),
                PlayerEv::VoteStart(_) => todo!(),
                PlayerEv::VoteCast(_) => todo!(),
                PlayerEv::VoteFail(_) => todo!(),
                PlayerEv::VoteSuccess(_) => todo!(),
                PlayerEv::Debug(_) => Msg::Nop,
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
