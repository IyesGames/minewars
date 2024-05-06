//! MineWars protocol message stream codec

use mw_common::{grid::Pos, plid::PlayerId};
use thiserror::Error;
use num_traits::FromPrimitive;

use crate::msg::*;

#[derive(Debug, Error)]
pub enum MsgBinWriteError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Invalid value: {0}")]
    InvalidValue(u32),
}

#[derive(Debug, Error)]
pub enum MsgBinReadError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown Instruction: {0}")]
    UnknownOp(u8),
    #[error("Invalid value: {0}")]
    InvalidValue(u32),
}

pub struct MsgBinRead {
}
pub struct MsgBinWrite {
}

impl MsgWriter for MsgBinWrite {
    type Error = MsgBinWriteError;
    fn write<W: std::io::Write>(&mut self, w: &mut W, msgs: &[Msg], max_bytes: usize) -> Result<(usize, usize), Self::Error> {
        if max_bytes == 0 {
            return Ok((0, 0));
        }
        let Some(first) = msgs.get(0) else {
            return Ok((0, 0));
        };
        let mut n_msgs = 1;
        let mut n_bytes = 0;
        match first {
            Msg::Nop => {},
            Msg::Player { plid, subplid, status } => {
                let (n_bytes, byte_status) = match status {
                    MsgPlayer::Joined =>                 (3, 0b00000000),
                    MsgPlayer::NetRttInfo { .. } =>      (4, 0b00000001),
                    MsgPlayer::Timeout { .. } =>         (4, 0b00000010),
                    MsgPlayer::TimeoutFinished =>        (3, 0b00000011),
                    MsgPlayer::Exploded { .. } =>        (6, 0b00000100),
                    MsgPlayer::LivesRemain { .. } =>     (4, 0b00000101),
                    MsgPlayer::Protected =>              (3, 0b00000110),
                    MsgPlayer::Unprotected =>            (3, 0b00000111),
                    MsgPlayer::Eliminated =>             (3, 0b00001000),
                    MsgPlayer::Surrendered =>            (3, 0b00001001),
                    MsgPlayer::Disconnected =>           (3, 0b00001010),
                    MsgPlayer::Kicked =>                 (3, 0b00001011),
                    MsgPlayer::MatchTimeRemain { .. } => (5, 0b00010011),
                };
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte1 = (u8::from(*plid) & 0x0F) | if let Some(x) = subplid {
                    (x & 0x0F) << 4
                } else {
                    0xF0
                };
                w.write_all(&[0b00000000, byte1, byte_status])?;
                match status {
                    MsgPlayer::NetRttInfo { duration } => w.write_all(&[duration.0])?,
                    MsgPlayer::Timeout { duration } => w.write_all(&[duration.0])?,
                    MsgPlayer::Exploded { pos, killer } => w.write_all(&[pos.y() as u8, pos.x() as u8, u8::from(*killer)])?,
                    MsgPlayer::LivesRemain { lives } => w.write_all(&[*lives])?,
                    MsgPlayer::MatchTimeRemain { secs } => w.write_all(&secs.to_be_bytes())?,
                    _ => {}
                }
            },
            Msg::Tremor => {
                n_bytes = 1;
                w.write_all(&[0b00000001])?;
            },
            Msg::Smoke { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000010, pos.y() as u8, pos.x() as u8])?;
            },
            Msg::Unsmoke { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000011, pos.y() as u8, pos.x() as u8])?;
            },
            Msg::CitMoney { cit, money } => {
                n_bytes = 6;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                if *money >= (1 << 31) {
                    return Err(MsgBinWriteError::InvalidValue(*money));
                }
                w.write_all(&[0b00000100, *cit])?;
                w.write_all(&money.to_be_bytes())?;
            },
            Msg::CitIncome { cit, money, income } => {
                n_bytes = 8;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                if *money >= (1 << 31) {
                    return Err(MsgBinWriteError::InvalidValue(*money));
                }
                let money = (1 << 31) | money;
                w.write_all(&[0b00000100, *cit])?;
                w.write_all(&money.to_be_bytes())?;
                w.write_all(&income.to_be_bytes())?;
            },
            Msg::CitMoneyTransact { cit, amount } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000101, *cit])?;
                w.write_all(&amount.to_be_bytes())?;
            },
            Msg::CitRes { cit, res } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000110, *cit])?;
                w.write_all(&res.to_be_bytes())?;
            },
            Msg::CitTradeInfo { cit, export, import } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000111, *cit, *export, *import])?;
            },
            Msg::Flag { plid, pos } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00001111, u8::from(*plid), pos.y() as u8, pos.x() as u8])?;
            },
            Msg::StructureGone { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00100000, pos.y() as u8, pos.x() as u8])?;
            },
            Msg::StructureHp { pos, hp } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                if *hp > 0x0F {
                    return Err(MsgBinWriteError::InvalidValue(*hp as u32));
                }
                let byte0 = 0b00100000 | *hp;
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
            },
            Msg::BuildNew { pos, kind, pts } => {
                n_bytes = 5;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b01000000 | (*kind as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                w.write_all(&pts.to_be_bytes())?;
            },
            Msg::Construction { pos, current, rate } => {
                n_bytes = 7;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b01001111, pos.y() as u8, pos.x() as u8])?;
                w.write_all(&current.to_be_bytes())?;
                w.write_all(&rate.to_be_bytes())?;
            },
            Msg::RevealStructure { pos, kind } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b01010000 | (*kind as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
            },
            Msg::RevealItem { pos, item } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b00010000 | (*item as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
            },
            Msg::TileKind { pos, kind } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b01110000 | (*kind as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
            },
            Msg::TileOwner { pos, plid } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let mut byte0 = 0b10000000 | ((u8::from(*plid) & 0x0F) << 3);
                let mut tilecount = 0;
                for msg in msgs.iter().skip(1) {
                    if tilecount >= 7 {
                        break;
                    }
                    if let Msg::TileOwner { plid: plid2, pos: _ } = msg {
                        if plid2 != plid {
                            break;
                        }
                        if max_bytes >= n_bytes + 2 {
                            tilecount += 1;
                            n_bytes += 2;
                            continue;
                        }
                    }
                    break;
                }
                byte0 |= tilecount;
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                for msg in msgs.iter().skip(1).take(tilecount as usize) {
                    let Msg::TileOwner { pos, plid: _ } = msg else {
                        unreachable!();
                    };
                    w.write_all(&[pos.y() as u8, pos.x() as u8])?;
                }
                n_msgs += tilecount as usize;
            },
            Msg::Explode { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let mut byte0 = 0b00100000;
                let mut tilecount = 0;
                for msg in msgs.iter().skip(1) {
                    if tilecount >= 15 {
                        break;
                    }
                    if let Msg::Explode { .. } = msg {
                        if max_bytes >= n_bytes + 2 {
                            tilecount += 1;
                            n_bytes += 2;
                            continue;
                        }
                    }
                    break;
                }
                byte0 |= tilecount;
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                for msg in msgs.iter().skip(1).take(tilecount as usize) {
                    let Msg::Explode { pos } = msg else {
                        unreachable!();
                    };
                    w.write_all(&[pos.y() as u8, pos.x() as u8])?;
                }
                n_msgs += tilecount as usize;
            },
            Msg::DigitCapture { pos, digit, asterisk } => {
                n_bytes = 4;
                let mut tilecount = 0;
                for msg in msgs.iter().skip(1) {
                    if tilecount >= 15 {
                        break;
                    }
                    if let Msg::DigitCapture { .. } = msg {
                        let needed_bytes = 2 + (tilecount & 1) as usize;
                        if max_bytes >= n_bytes + needed_bytes {
                            tilecount += 1;
                            n_bytes += needed_bytes;
                            continue;
                        }
                    }
                    break;
                }
                if tilecount == 0 {
                    // single encoding
                    n_bytes = 3;
                    if max_bytes < n_bytes {
                        return Ok((0, 0));
                    }
                    let mut byte0 = 0b01100000;
                    byte0 |= (*asterisk as u8) << 3;
                    byte0 |= *digit & 0x07;
                    w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                } else {
                    // multi encoding
                    n_msgs += tilecount as usize;
                    if max_bytes < n_bytes {
                        return Ok((0, 0));
                    }
                    let mut byte0 = 0b10000000;
                    byte0 |= tilecount;
                    w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                    for msg in msgs.iter().skip(1).take(tilecount as usize) {
                        let Msg::DigitCapture { pos, .. } = msg else {
                            unreachable!();
                        };
                        w.write_all(&[pos.y() as u8, pos.x() as u8])?;
                    }
                    let mut high = false;
                    let mut digbyte = 0;
                    digbyte |= (*asterisk as u8) << 7;
                    digbyte |= (*digit & 0x07) << 4;
                    for msg in msgs.iter().skip(1).take(tilecount as usize) {
                        let Msg::DigitCapture { asterisk, digit, .. } = msg else {
                            unreachable!();
                        };
                        if high {
                            digbyte |= (*asterisk as u8) << 7;
                            digbyte |= (*digit & 0x07) << 4;
                            high = false;
                        } else {
                            digbyte |= (*asterisk as u8) << 3;
                            digbyte |= (*digit & 0x07) << 0;
                            w.write_all(&[digbyte])?;
                            digbyte = 0;
                            high = true;
                        }
                    }
                    if !high {
                        w.write_all(&[digbyte])?;
                    }
                }
            },
        }
        Ok((n_msgs, n_bytes))
    }
}

impl MsgReader for MsgBinRead {
    type Error = MsgBinReadError;
    fn read<R: std::io::BufRead>(&mut self, r: &mut R, out: &mut Vec<Msg>) -> Result<usize, Self::Error> {
        let mut byte0 = [0u8];
        r.read_exact(&mut byte0)?;
        let byte0 = byte0[0];

        if byte0 == 0b00000001 {
            out.push(Msg::Tremor);
            return Ok(1);
        }
        if byte0 & 0b11111110 == 0b00000010 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            if byte0 & 1 == 0 {
                out.push(Msg::Smoke { pos });
            } else {
                out.push(Msg::Unsmoke { pos });
            }
            return Ok(1);
        }
        if byte0 == 0b00000100 {
            let mut bytes = [0; 5];
            r.read_exact(&mut bytes)?;
            let cit = bytes[0];
            let money = u32::from_be_bytes([
                bytes[1], bytes[2], bytes[3], bytes[4]
            ]);
            if money & (1 << 31) != 0 {
                let mut bytes = [0; 2];
                r.read_exact(&mut bytes)?;
                let income = u16::from_be_bytes([
                    bytes[0], bytes[1]
                ]);
                out.push(Msg::CitIncome { cit, money, income });
            } else {
                out.push(Msg::CitMoney { cit, money });
            }
            return Ok(1);
        }
        if byte0 == 0b00000101 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let cit = bytes[0];
            let amount = i16::from_be_bytes([
                bytes[1], bytes[2]
            ]);
            out.push(Msg::CitMoneyTransact { cit, amount });
            return Ok(1);
        }
        if byte0 == 0b00000110 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let cit = bytes[0];
            let res = u16::from_be_bytes([
                bytes[1], bytes[2]
            ]);
            out.push(Msg::CitRes { cit, res });
            return Ok(1);
        }
        if byte0 == 0b00000111 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let cit = bytes[0];
            let export = bytes[1];
            let import = bytes[2];
            out.push(Msg::CitTradeInfo { cit, export, import });
            return Ok(1);
        }
        if byte0 == 0b00001111 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let plid = PlayerId::from(bytes[0]);
            let mut pos = Pos::default();
            pos.set_y(bytes[1] as i8);
            pos.set_x(bytes[2] as i8);
            out.push(Msg::Flag { plid, pos });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b00100000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let hp = byte0 & 0b00001111;
            if hp > 0 {
                out.push(Msg::StructureHp { pos, hp });
            } else {
                out.push(Msg::StructureGone { pos });
            }
            return Ok(1);
        }
        if byte0 == 0b01001111 {
            let mut bytes = [0; 6];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let current = u16::from_be_bytes([
                bytes[2], bytes[3]
            ]);
            let rate = u16::from_be_bytes([
                bytes[4], bytes[5]
            ]);
            out.push(Msg::Construction { pos, current, rate });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b01000000 {
            let mut bytes = [0; 4];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let pts = u16::from_be_bytes([
                bytes[2], bytes[3]
            ]);
            let kind_byte = byte0 & 0x0F;
            if kind_byte == 0x0F {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            }
            let Some(kind) = MsgStructureKind::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(Msg::BuildNew { pos, kind, pts });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b01010000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let kind_byte = byte0 & 0x0F;
            if kind_byte == 0x0F {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            }
            let Some(kind) = MsgStructureKind::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(Msg::RevealStructure { pos, kind });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b00010000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let kind_byte = byte0 & 0x0F;
            let Some(item) = MsgItem::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(Msg::RevealItem { pos, item });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b01110000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let kind_byte = byte0 & 0x0F;
            let Some(kind) = MsgTileKind::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(Msg::TileKind { pos, kind });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b01100000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let asterisk = byte0 & 0b00001000 != 0;
            let digit = byte0 & 0b00000111;
            out.push(Msg::DigitCapture { pos, digit, asterisk });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b00110000 {
            let mut bytes = [0; 2 * 16];
            let n_tiles = ((byte0 & 0b00001111) + 1) as usize;
            r.read_exact(&mut bytes[..(2 * n_tiles)])?;
            for i in 0..n_tiles {
                let mut pos = Pos::default();
                pos.set_y(bytes[i * 2 + 0] as i8);
                pos.set_x(bytes[i * 2 + 1] as i8);
                out.push(Msg::Explode { pos });
            }
            return Ok(n_tiles);
        }
        if byte0 & 0b11110000 == 0b10000000 {
            let mut bytes = [0; 2 * 16 + 8];
            let n_tiles = ((byte0 & 0b00001111) + 1) as usize;
            let n_bytes = n_tiles * 2 + (n_tiles + 1) / 2;
            r.read_exact(&mut bytes[..n_bytes])?;
            for i in 0..n_tiles {
                let mut pos = Pos::default();
                pos.set_y(bytes[i * 2 + 0] as i8);
                pos.set_x(bytes[i * 2 + 1] as i8);
                let off_digit = n_tiles * 2 + i / 2;
                let (asterisk, digit) = if i / 2 == 0 {
                    (
                        bytes[off_digit] & 0b10000000 != 0,
                        (bytes[off_digit] & 0b01110000) >> 4,
                    )
                } else {
                    (
                        bytes[off_digit] & 0b00001000 != 0,
                        bytes[off_digit] & 0b00000111,
                    )
                };
                out.push(Msg::DigitCapture { pos, digit, asterisk });
            }
            return Ok(n_tiles);
        }
        if byte0 & 0b10000000 == 0b10000000 {
            let mut bytes = [0; 2 * 8];
            let plid = PlayerId::from((byte0 & 0b01111000) >> 3);
            let n_tiles = ((byte0 & 0b00000111) + 1) as usize;
            r.read_exact(&mut bytes[..(2 * n_tiles)])?;
            for i in 0..n_tiles {
                let mut pos = Pos::default();
                pos.set_y(bytes[i * 2 + 0] as i8);
                pos.set_x(bytes[i * 2 + 1] as i8);
                out.push(Msg::TileOwner { pos, plid });
            }
            return Ok(n_tiles);
        }

        Err(MsgBinReadError::UnknownOp(byte0))
    }
}
