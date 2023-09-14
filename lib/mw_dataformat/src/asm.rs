use std::fmt::Formatter;
use mw_common::{grid::Pos, plid::PlayerId};
use thiserror::Error;

use crate::msg::{Msg, MsgItem, MsgStructureKind};

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
                writeln!(fmt, "PLAYER {} {}", u8::from(*plid), status)?;
            },
            Msg::Capture { pos, digit } => {
                writeln!(fmt, "DIGITS {}/{},{}", digit, pos.0, pos.1)?;
            },
            Msg::TileOwner { pos, plid } => {
                writeln!(fmt, "OWNER {} {},{}", u8::from(*plid), pos.0, pos.1)?;
            },
            Msg::Digit { pos, digit } => {
                writeln!(fmt, "DIGIT {}/{},{}", digit, pos.0, pos.1)?;
            },
            Msg::CitUpdate { cit, money, income, res } => {
                writeln!(fmt, "CIT {} {} {} {}", cit, res, money, income)?;
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
            Msg::RevealItem { pos, item } => {
                writeln!(fmt, "ITEM {},{} {}", pos.0, pos.1, match item {
                    MsgItem::None => "none",
                    MsgItem::Decoy => "decoy",
                    MsgItem::Mine => "mine",
                    MsgItem::Flash => "flash",
                })?;
            },
            Msg::Explode { pos } => {
                writeln!(fmt, "EXPLODE {},{}", pos.0, pos.1)?;
            },
            Msg::Smoke { pos } => {
                writeln!(fmt, "SMOKE {},{}", pos.0, pos.1)?;
            },
            Msg::Tremor => {
                writeln!(fmt, "SHAKE")?;
            },
            Msg::Nop => {
                writeln!(fmt, "NOP")?;
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
                let Some(arg_plid) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_status) = components.next() else {
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
                let Ok(status) = arg_status.parse() else {
                    return Err(MsgAsmError::BadArg(arg_status.to_owned()));
                };
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Player {
                    plid, status,
                };
                Ok(1)
            }
            "DIGITS" => {
                let mut n = 0;
                for arg in components {
                    let Some((arg_digit, arg_pos)) = arg.split_once('/') else {
                        return Err(MsgAsmError::BadArg(arg.to_owned()));
                    };
                    let Ok(digit) = arg_digit.parse() else {
                        return Err(MsgAsmError::BadArg(arg_digit.to_owned()));
                    };
                    let pos = parse_pos(arg_pos)?;
                    if buffer.len() < n + 1 {
                        return Err(MsgAsmError::BufferFull);
                    }
                    buffer[n] = Msg::Capture {
                        digit, pos,
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
            "DIGIT" => {
                let Some(arg) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some((arg_digit, arg_pos)) = arg.split_once('/') else {
                    return Err(MsgAsmError::BadArg(arg.to_owned()));
                };
                if components.next().is_some() {
                    return Err(MsgAsmError::TooManyArgs);
                }
                let Ok(digit) = arg_digit.parse() else {
                    return Err(MsgAsmError::BadArg(arg_digit.to_owned()));
                };
                let pos = parse_pos(arg_pos)?;
                if buffer.len() < 1 {
                    return Err(MsgAsmError::BufferFull);
                }
                buffer[0] = Msg::Digit {
                    digit, pos,
                };
                Ok(1)
            }
            "CIT" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmError::NotEnoughArgs);
                };
                let Some(arg_res) = components.next() else {
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
                let Ok(res) = arg_res.parse() else {
                    return Err(MsgAsmError::BadArg(arg_res.to_owned()));
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
                buffer[0] = Msg::CitUpdate {
                    cit, res, money, income,
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
                    "FLASH" => MsgItem::Flash,
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_asm() {
        let source = "
            nop
            SHAKE ; comment
            DIGIT 4/0,0
            OWNER 3 4,5 7,8 ; varargs
            DIGITS 1/1,1 0/-1,-2 3/6,7
            item 3,-4 None
            ITEM 3,-3 Flash
            ITEM 3,-2 mine
            ITEM 3,-1 DECOY
            Explode 3,-1
            EXPLODE 7,8 8,9 -1,-2
            CIT 0 200 1503 111
            STRUCT 10,11 tower
            STRUCT -10,31 ROAD
            STRUCT 15,-31 Wall
            BUILDNEW 0,1 bridge 420
            StructHp 10,11 5
            DECONSTRUCT 0,1
            PLAYER 5 0
            BUILD 0,1 123 42
            SMOKE 0,0
        ";
        let output = &[
            Msg::Nop,
            Msg::Tremor,
            Msg::Digit { digit: 4, pos: Pos(0,0) },
            Msg::TileOwner { plid: 3.into(), pos: Pos(4,5) },
            Msg::TileOwner { plid: 3.into(), pos: Pos(7,8) },
            Msg::Capture { digit: 1, pos: Pos(1,1) },
            Msg::Capture { digit: 0, pos: Pos(-1,-2) },
            Msg::Capture { digit: 3, pos: Pos(6,7) },
            Msg::RevealItem { pos: Pos(3,-4), item: MsgItem::None },
            Msg::RevealItem { pos: Pos(3,-3), item: MsgItem::Flash },
            Msg::RevealItem { pos: Pos(3,-2), item: MsgItem::Mine },
            Msg::RevealItem { pos: Pos(3,-1), item: MsgItem::Decoy },
            Msg::Explode { pos: Pos(3, -1) },
            Msg::Explode { pos: Pos(7, 8) },
            Msg::Explode { pos: Pos(8, 9) },
            Msg::Explode { pos: Pos(-1, -2) },
            Msg::CitUpdate { cit: 0, res: 200, money: 1503, income: 111 },
            Msg::RevealStructure { pos: Pos(10, 11), kind: MsgStructureKind::Tower },
            Msg::RevealStructure { pos: Pos(-10, 31), kind: MsgStructureKind::Road },
            Msg::RevealStructure { pos: Pos(15, -31), kind: MsgStructureKind::Wall },
            Msg::BuildNew { pos: Pos(0, 1), kind: MsgStructureKind::Bridge, pts: 420 },
            Msg::StructureHp { pos: Pos(10, 11), hp: 5 },
            Msg::StructureGone { pos: Pos(0, 1) },
            Msg::Player { plid: 5.into(), status: 0 },
            Msg::Construction { pos: Pos(0, 1), current: 123, rate: 42 },
            Msg::Smoke { pos: Pos(0, 0) },
        ];
        let mut buffer = vec![Msg::Nop; 64];
        let len = Msg::asm_all(source, &mut buffer)
            .expect("asm unsuccessful");
        eprintln!("{:#?}", output);
        eprintln!("{:#?}", &buffer[..len]);
        assert_eq!(&buffer[..len], output);
    }
}
