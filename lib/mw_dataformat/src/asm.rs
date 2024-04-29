//! "Assembly Language" for Game Event Messages
//!
//! The idea is to help with debugging and development of the MineWars
//! data stream by having a human-readable representation of the binary data.

use std::fmt::Formatter;
use mw_common::{grid::Pos, plid::PlayerId};
use thiserror::Error;

use crate::msg::{Msg, MsgItem, MsgStructureKind, MsgTileKind};

pub trait Assembly: Sized {
    type DisasmError: std::error::Error;
    type AsmError: std::error::Error;

    /// Attempt to disassemble one instruction
    ///
    /// `buffer` may contain data that represents any (possibly unknown) number of
    /// instructions. This method will interpret the first, and return how many
    /// elements in the buffer were processed. The returned `usize` represents
    /// the offset at which the next instruction is supposed to start. Advance the
    /// slice to that offset and call this method again, to disassemble the next
    /// instruction.
    fn disasm_one(buffer: &[Self], fmt: &mut Formatter) -> Result<usize, Self::DisasmError>;

    /// Attempt to disassemble an entire sequence of instructions
    ///
    /// `buffer` may contain data that represents any (possibly unknown) number of
    /// instructions. This method will attempt to process all of it. It will keep going
    /// until it either disassembles everything successfully, or encounters an error.
    ///
    /// Returns the total number of instructions that have been disassembled, as well as
    /// a `Result` indicating whether that is all of them (the entire buffer has been processed),
    /// or disassembly stopped at some point due to an error. If there is an error, the
    /// offset within the buffer where the error occured is returned alongside the error.
    fn disasm_all(buffer: &[Self], fmt: &mut Formatter) -> (usize, Result<(), (usize, Self::DisasmError)>) {
        let mut current_offset = 0;
        let mut total_count = 0;
        while current_offset < buffer.len() {
            let remaining_data = &buffer[current_offset..];
            match Self::disasm_one(remaining_data, fmt) {
                Ok(offset) => {
                    current_offset += offset;
                    total_count += 1;
                }
                Err(e) => {
                    return (total_count, Err((current_offset, e)));
                }
            }
        }
        (total_count, Ok(()))
    }

    /// Attempt to assemble one instruction
    ///
    /// `source` is the source code text string. If it contains more than one instruction,
    /// parsing will stop after the first. Given that our format is newline-delimited,
    /// you should probably only call this method on individual lines.
    ///
    /// On success, returns the number of elements written into `buffer`. You should advance
    /// by that offset before any future write to the buffer, to not clobber the data
    /// written by this method.
    fn asm_one(source: &str, buffer: &mut [Self]) -> Result<usize, Self::AsmError>;

    /// Attempt to assemble an entire sequence of instructions
    ///
    /// `source` is the source code text string. If it contains more than one instruction,
    /// it will keep going and process as much as possible. It is expected to be newline-delimited,
    /// where each line represents one instruction (or is blank).
    ///
    /// On success, returns the number of elements written into `buffer`. You should advance
    /// by that offset before any future write to the buffer, to not clobber the data
    /// written by this method.
    fn asm_all(source: &str, buffer: &mut [Self]) -> Result<usize, Self::AsmError> {
        let mut current_offset = 0;
        for line in source.lines() {
            // will panic if overrun; we expect `asm_one` to not cause
            // buffer overruns; it should return an error if there is no
            // space in the buffer
            let remaining_buffer = &mut buffer[current_offset..];
            let offset = Self::asm_one(line, remaining_buffer)?;
            current_offset += offset;
        }
        Ok(current_offset)
    }
}

#[derive(Debug, Error)]
pub enum MsgDisasmError {
    #[error("Formatter: {0}")]
    Fmt(#[from] std::fmt::Error),
}

#[derive(Debug, Error)]
pub enum MsgAsmError {
    #[error("No space in output buffer.")]
    BufferFull,
    #[error("Unknown Instruction: {0}")]
    UnknownOp(String),
    #[error("Invalid Operand: {0}")]
    BadArg(String),
    #[error("Expected more operands!")]
    NotEnoughArgs,
    #[error("Too many operands!")]
    TooManyArgs,
}

impl Assembly for Msg {
    type DisasmError = MsgDisasmError;
    type AsmError = MsgAsmError;

    fn disasm_one(buffer: &[Self], fmt: &mut Formatter) -> Result<usize, Self::DisasmError> {
        let Some(first) = buffer.get(0) else {
            return Ok(0);
        };

        match first {
            Msg::Player { plid, status } => {
                // writeln!(fmt, "PLAYER {} {}", u8::from(*plid), status)?;
                todo!()
            },
            Msg::DigitCapture { pos, digit, asterisk } => {
                writeln!(fmt, "DIGITS {}{}/{},{}", digit, if *asterisk { "*" } else { "" }, pos.0, pos.1)?;
            },
            Msg::TileOwner { pos, plid } => {
                writeln!(fmt, "OWNER {} {},{}", u8::from(*plid), pos.0, pos.1)?;
            },
            Msg::CitRes { cit, res } => {
                writeln!(fmt, "CITRES {} {}", cit, res)?;
            },
            Msg::CitMoneyTransact { cit, amount } => {
                writeln!(fmt, "CITTRANS {} {}", cit, amount)?;
            },
            Msg::CitMoney { cit, money } => {
                writeln!(fmt, "CITMONEY {} {}", cit, money)?;
            },
            Msg::CitIncome { cit, money, income } => {
                writeln!(fmt, "CITINCOME {} {} {}", cit, money, income)?;
            },
            Msg::CitTradeInfo { cit, export, import } => {
                writeln!(fmt, "CITTRADE {} {} {}", cit, export, import)?;
            },
            Msg::RevealStructure { pos, kind } => {
                writeln!(fmt, "STRUCT {},{} {}", pos.0, pos.1, match kind {
                    MsgStructureKind::Road => "road",
                    MsgStructureKind::Bridge => "bridge",
                    MsgStructureKind::Wall => "wall",
                    MsgStructureKind::Tower => "tower",
                })?;
            },
            Msg::StructureGone { pos } => {
                writeln!(fmt, "DECONSTRUCT {},{}", pos.0, pos.1)?;
            },
            Msg::StructureHp { pos, hp } => {
                writeln!(fmt, "STRUCTHP {},{} {}", pos.0, pos.1, hp)?;
            },
            Msg::BuildNew { pos, kind, pts } => {
                writeln!(fmt, "BUILDNEW {},{} {} {}", pos.0, pos.1, match kind {
                    MsgStructureKind::Road => "road",
                    MsgStructureKind::Bridge => "bridge",
                    MsgStructureKind::Wall => "wall",
                    MsgStructureKind::Tower => "tower",
                }, pts)?;
            },
            Msg::Construction { pos, current, rate } => {
                writeln!(fmt, "BUILD {},{} {} {}", pos.0, pos.1, current, rate)?;
            },
            Msg::RevealItem { pos, item }  => {
                writeln!(fmt, "ITEM {},{} {}", pos.0, pos.1, match item {
                    MsgItem::None => "none",
                    MsgItem::Decoy => "decoy",
                    MsgItem::Mine => "mine",
                    MsgItem::Trap => "trap",
                })?;
            },
            Msg::Explode { pos } => {
                writeln!(fmt, "EXPLODE {},{}", pos.0, pos.1)?;
            },
            Msg::Smoke { pos } => {
                writeln!(fmt, "SMOKE {},{}", pos.0, pos.1)?;
            },
            Msg::Unsmoke { pos } => {
                writeln!(fmt, "UNSMOKE {},{}", pos.0, pos.1)?;
            },
            Msg::Tremor => {
                writeln!(fmt, "SHAKE")?;
            },
            Msg::Nop => {
                writeln!(fmt, "NOP")?;
            },
            Msg::PlayerSub { plid, subplid, status } => todo!(),
            Msg::Flag { plid, pos } => {
                writeln!(fmt, "FLAG {} {},{}", u8::from(*plid), pos.0, pos.1)?;
            },
            Msg::TileKind { pos, kind } => {
                writeln!(fmt, "TILE {},{} {}", pos.0, pos.1, match kind {
                    MsgTileKind::Water => "water",
                    MsgTileKind::Regular => "regular",
                    MsgTileKind::Fertile => "fertile",
                    MsgTileKind::Foundation => "foundation",
                    MsgTileKind::Destroyed => "destroyed",
                    MsgTileKind::Mountain => "mountain",
                    MsgTileKind::Forest => "forest",
                })?;
            },
        }

        Ok(1)
    }
    fn asm_one(source: &str, buffer: &mut [Self]) -> Result<usize, Self::AsmError> {
        // get the part before any comment and trim whitespace
        if source.is_empty() {
            return Ok(0);
        }
        let Some(source) = source.split(';').next() else {
            return Ok(0);
        };
        let source = source.trim();
        if source.is_empty() {
            return Ok(0);
        }
        let mut components = source.split_ascii_whitespace();
        let Some(iname) = components.next() else {
            return Ok(0);
        };
        if source.is_empty() {
            return Ok(0);
        }

        match iname.to_ascii_uppercase().as_str() {
            "PLAYER" => {
                todo!()
                // let Some(arg_plid) = components.next() else {
                //     return Err(MsgAsmError::NotEnoughArgs);
                // };
                // let Some(arg_status) = components.next() else {
                //     return Err(MsgAsmError::NotEnoughArgs);
                // };
                // if components.next().is_some() {
                //     return Err(MsgAsmError::TooManyArgs);
                // }
                // let Ok(plid) = arg_plid.parse::<u8>() else {
                //     return Err(MsgAsmError::BadArg(arg_plid.to_owned()));
                // };
                // if plid > 7 {
                //     return Err(MsgAsmError::BadArg(arg_plid.to_owned()));
                // }
                // let plid = PlayerId::from(plid);
                // let Ok(status) = arg_status.parse() else {
                //     return Err(MsgAsmError::BadArg(arg_status.to_owned()));
                // };
                // if buffer.len() < 1 {
                //     return Err(MsgAsmError::BufferFull);
                // }
                // buffer[0] = Msg::Player {
                //     plid, status,
                // };
                // Ok(1)
            }
            "TILE" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_kind) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let kind = match arg_kind.to_ascii_uppercase().as_str() {
                    "WATER" => MsgTileKind::Water,
                    "REGULAR" => MsgTileKind::Regular,
                    "FERTILE" => MsgTileKind::Fertile,
                    "DESTROYED" => MsgTileKind::Destroyed,
                    "FOUNDATION" => MsgTileKind::Foundation,
                    "MOUNTAIN" => MsgTileKind::Mountain,
                    "FOREST" => MsgTileKind::Forest,
                    other => {
                        return Err(MsgAsmError::BadArg(other.to_owned()));
                    }
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::TileKind {
                    pos, kind,
                };
                Ok(1)
            }
            "DIGITS" => {
                let mut n = 0;
                for arg in components {
                    let Some((mut arg_digit, arg_pos)) = arg.split_once('/') else {
                        return Err(MsgAsmError::BadArg(arg.to_owned()));
                    };
                    let asterisk = if let Some(s) = arg_digit.strip_suffix('*') {
                        arg_digit = s;
                        true
                    } else {
                        false
                    };
                    let Ok(digit) = arg_digit.parse() else {
                        return Err(MsgAsmError::BadArg(arg_digit.to_owned()));
                    };
                    let pos = parse_pos(arg_pos)?;
                    if buffer.len() < n + 1 {
                        return Err(MsgAsmError::BufferFull);
                    }
                    buffer[n] = Msg::DigitCapture {
                        digit, pos, asterisk,
                    };
                    n += 1;
                }
                if n == 0 {
                    return Err(MsgAsmError::NotEnoughArgs);
                }
                Ok(n)
            }
            "OWNER" => {
                let Some(arg_plid) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Ok(plid) = arg_plid.parse::<u8>() else {
                    return Err(MsgAsmError::BadArg(arg_plid.to_owned()));
                };
                if plid > 7 {
                    return Err(MsgAsmError::BadArg(arg_plid.to_owned()));
                }
                let plid = PlayerId::from(plid);
                let mut n = 0;
                for arg in components {
                    let pos = parse_pos(arg)?;
                    if buffer.len() < n + 1 {
                        return Err(MsgAsmError::BufferFull);
                    }
                    buffer[n] = Msg::TileOwner {
                        plid, pos,
                    };
                    n += 1;
                }
                if n == 0 {
                    return Err(MsgAsmError::NotEnoughArgs);
                }
                Ok(n)
            }
            "FLAG" => {
                let Some(arg_plid) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let Ok(plid) = arg_plid.parse::<u8>() else {
                    return Err(MsgAsmError::BadArg(arg_plid.to_owned()));
                };
                if plid > 7 {
                    return Err(MsgAsmError::BadArg(arg_plid.to_owned()));
                }
                let plid = PlayerId::from(plid);
                let pos = parse_pos(arg_pos)?;
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Flag {
                    pos, plid,
                };
                Ok(1)
            }
            "CITRES" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_res) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmError::BadArg(arg_cit.to_owned()));
                };
                let Ok(res) = arg_res.parse() else {
                    return Err(MsgAsmError::BadArg(arg_res.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::CitRes {
                    cit, res,
                };
                Ok(1)
            }
            "CITTRANS" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_amount) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmError::BadArg(arg_cit.to_owned()));
                };
                let Ok(amount) = arg_amount.parse() else {
                    return Err(MsgAsmError::BadArg(arg_amount.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::CitMoneyTransact {
                    cit, amount,
                };
                Ok(1)
            }
            "CITMONEY" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_money) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmError::BadArg(arg_cit.to_owned()));
                };
                let Ok(money) = arg_money.parse() else {
                    return Err(MsgAsmError::BadArg(arg_money.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::CitMoney {
                    cit, money,
                };
                Ok(1)
            }
            "CITINCOME" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_money) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_income) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmError::BadArg(arg_cit.to_owned()));
                };
                let Ok(money) = arg_money.parse() else {
                    return Err(MsgAsmError::BadArg(arg_money.to_owned()));
                };
                let Ok(income) = arg_income.parse() else {
                    return Err(MsgAsmError::BadArg(arg_income.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::CitIncome {
                    cit, money, income,
                };
                Ok(1)
            }
            "CITTRADE" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_export) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_import) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmError::BadArg(arg_cit.to_owned()));
                };
                let Ok(export) = arg_export.parse() else {
                    return Err(MsgAsmError::BadArg(arg_export.to_owned()));
                };
                let Ok(import) = arg_import.parse() else {
                    return Err(MsgAsmError::BadArg(arg_import.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::CitTradeInfo {
                    cit, export, import,
                };
                Ok(1)
            }
            "STRUCT" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_kind) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let kind = match arg_kind.to_ascii_uppercase().as_str() {
                    "ROAD" => MsgStructureKind::Road,
                    "BRIDGE" => MsgStructureKind::Bridge,
                    "WALL" => MsgStructureKind::Wall,
                    "TOWER" => MsgStructureKind::Tower,
                    other => {
                        return Err(MsgAsmError::BadArg(other.to_owned()));
                    }
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::RevealStructure {
                    pos, kind,
                };
                Ok(1)
            }
            "DECONSTRUCT" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::StructureGone {
                    pos
                };
                Ok(1)
            }
            "STRUCTHP" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_hp) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let Ok(hp) = arg_hp.parse() else {
                    return Err(MsgAsmError::BadArg(arg_hp.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::StructureHp {
                    pos, hp,
                };
                Ok(1)
            }
            "BUILDNEW" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_kind) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_pts) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let kind = match arg_kind.to_ascii_uppercase().as_str() {
                    "ROAD" => MsgStructureKind::Road,
                    "BRIDGE" => MsgStructureKind::Bridge,
                    "WALL" => MsgStructureKind::Wall,
                    "TOWER" => MsgStructureKind::Tower,
                    other => {
                        return Err(MsgAsmError::BadArg(other.to_owned()));
                    }
                };
                let Ok(pts) = arg_pts.parse() else {
                    return Err(MsgAsmError::BadArg(arg_pts.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::BuildNew {
                    pos, kind, pts,
                };
                Ok(1)
            }
            "BUILD" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_current) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_rate) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let Ok(current) = arg_current.parse() else {
                    return Err(MsgAsmError::BadArg(arg_current.to_owned()));
                };
                let Ok(rate) = arg_rate.parse() else {
                    return Err(MsgAsmError::BadArg(arg_current.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Construction {
                    pos, current, rate
                };
                Ok(1)
            }
            "ITEM" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_item) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let item = match arg_item.to_ascii_uppercase().as_str() {
                    "NONE" => MsgItem::None,
                    "DECOY" => MsgItem::Decoy,
                    "MINE" => MsgItem::Mine,
                    "TRAP" => MsgItem::Trap,
                    other => {
                        return Err(MsgAsmError::BadArg(other.to_owned()));
                    }
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::RevealItem {
                    pos, item
                };
                Ok(1)
            }
            "EXPLODE" => {
                let mut n = 0;
                for arg in components {
                    let pos = parse_pos(arg)?;
                    if buffer.len() < n + 1 {
                        return Err(MsgAsmError::BufferFull);
                    }
                    buffer[n] = Msg::Explode {
                        pos,
                    };
                    n += 1;
                }
                if n == 0 {
                    return Err(MsgAsmError::NotEnoughArgs);
                }
                Ok(n)
            }
            "SMOKE" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Smoke {
                    pos
                };
                Ok(1)
            }
            "UNSMOKE" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Unsmoke {
                    pos
                };
                Ok(1)
            }
            "SHAKE" => {
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Tremor;
                Ok(1)
            }
            "NOP" => {
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Nop;
                Ok(1)
            }
            other => {
                Err(MsgAsmError::UnknownOp(other.to_owned()))
            }
        }
    }
}

fn parse_pos(s: &str) -> Result<Pos, MsgAsmError> {
    // error we return if anything goes wrong
    let err = Err(MsgAsmError::BadArg(s.to_owned()));

    let mut parts = s.split(',');
    let Some(part_y) = parts.next() else {
        return err;
    };
    let Some(part_x) = parts.next() else {
        return err;
    };
    let Ok(y) = part_y.parse::<i8>() else {
        return err;
    };
    let Ok(x) = part_x.parse::<i8>() else {
        return err;
    };
    Ok(Pos(y,x))
}
