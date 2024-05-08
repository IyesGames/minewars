//! MineWars protocol message stream codec

use mw_common::{game::{event::MwEv, ItemKind, StructureKind, TileKind}, grid::Pos, plid::PlayerId, time::MwDur};
use thiserror::Error;
use num_traits::FromPrimitive;

use super::*;

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
    #[error("Unknown Player Instruction: {0}")]
    UnknownPlayerOp(u8),
    #[error("Invalid value: {0}")]
    InvalidValue(u32),
    #[error("String is not UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

pub struct MsgBinRead {
}
pub struct MsgBinWrite {
}

impl MsgBinRead {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl MsgBinWrite {
    pub fn new() -> Self {
        Self {
        }
    }
}

impl MsgWriter for MsgBinWrite {
    type Error = MsgBinWriteError;
    fn write<W: std::io::Write>(&mut self, w: &mut W, msgs: &[MwEv], max_bytes: usize) -> Result<(usize, usize), Self::Error> {
        if max_bytes == 0 {
            return Ok((0, 0));
        }
        let Some(first) = msgs.get(0) else {
            return Ok((0, 0));
        };
        let mut n_msgs = 1;
        let mut n_bytes = 0;
        match first {
            MwEv::Nop => {},
            MwEv::Debug(i, pos) => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00001110, *i, pos.y() as u8, pos.x() as u8])?;
            }
            MwEv::Player { plid, subplid, ev: status } => {
                let (byte_status, n_bytes, strlen) = match status {
                    PlayerEv::Joined { name } =>         (0b00000000, 4, name.floor_char_boundary(255)),
                    PlayerEv::NetRttInfo { .. } =>       (0b00000001, 4, 0),
                    PlayerEv::Timeout { .. } =>          (0b00000010, 3, 0),
                    PlayerEv::TimeoutFinished =>         (0b00000011, 6, 0),
                    PlayerEv::Exploded { .. } =>         (0b00000100, 4, 0),
                    PlayerEv::LivesRemain { .. } =>      (0b00000101, 3, 0),
                    PlayerEv::Protected =>               (0b00000110, 3, 0),
                    PlayerEv::Unprotected =>             (0b00000111, 3, 0),
                    PlayerEv::Eliminated =>              (0b00001000, 3, 0),
                    PlayerEv::Surrendered =>             (0b00001001, 3, 0),
                    PlayerEv::Disconnected =>            (0b00001010, 3, 0),
                    PlayerEv::Kicked =>                  (0b00001011, 5, 0),
                    PlayerEv::ChatAll { text } =>        (0b00010000, 4, text.floor_char_boundary(255)),
                    PlayerEv::ChatFriendly { text }  =>  (0b00010001, 4, text.floor_char_boundary(255)),
                    PlayerEv::MatchTimeRemain { .. } =>  (0b00010010, 4, 0),
                    PlayerEv::VoteNew { l10nkey, .. } => (0b00010011, 5, l10nkey.floor_char_boundary(255)),
                    PlayerEv::VoteNo { .. } =>           (0b00001100, 4, 0),
                    PlayerEv::VoteYes { .. } =>          (0b00001101, 4, 0),
                    PlayerEv::VoteFail { .. } =>         (0b00001110, 4, 0),
                    PlayerEv::VotePass { .. } =>         (0b00001111, 4, 0),
                };
                if max_bytes < n_bytes + strlen {
                    return Ok((0, 0));
                }
                let byte1 = (u8::from(*plid) & 0x0F) | if let Some(x) = subplid {
                    (x & 0x0F) << 4
                } else {
                    0xF0
                };
                w.write_all(&[0b00000000, byte1, byte_status])?;
                match status {
                    PlayerEv::Joined { name } => {
                        w.write_all(&[strlen as u8])?;
                        w.write_all(&name[..strlen].as_bytes())?;
                    }
                    PlayerEv::NetRttInfo { duration } => w.write_all(&[duration.0])?,
                    PlayerEv::Timeout { duration } => w.write_all(&[duration.0])?,
                    PlayerEv::Exploded { pos, killer } => w.write_all(&[pos.y() as u8, pos.x() as u8, u8::from(*killer)])?,
                    PlayerEv::LivesRemain { lives } => w.write_all(&[*lives])?,
                    PlayerEv::MatchTimeRemain { secs } => w.write_all(&secs.to_be_bytes())?,
                    PlayerEv::VoteNo { id } => w.write_all(&[*id])?,
                    PlayerEv::VoteYes { id } => w.write_all(&[*id])?,
                    PlayerEv::VoteFail { id } => w.write_all(&[*id])?,
                    PlayerEv::VotePass { id } => w.write_all(&[*id])?,
                    PlayerEv::ChatAll { text } => {
                        w.write_all(&[strlen as u8])?;
                        w.write_all(&text[..strlen].as_bytes())?;
                    }
                    PlayerEv::ChatFriendly { text } => {
                        w.write_all(&[strlen as u8])?;
                        w.write_all(&text[..strlen].as_bytes())?;
                    }
                    PlayerEv::VoteNew { id, l10nkey } => {
                        w.write_all(&[*id, strlen as u8])?;
                        w.write_all(&l10nkey[..strlen].as_bytes())?;
                    }
                    _ => {}
                }
            },
            MwEv::Tremor => {
                n_bytes = 1;
                w.write_all(&[0b00000001])?;
            },
            MwEv::Smoke { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000010, pos.y() as u8, pos.x() as u8])?;
            },
            MwEv::Unsmoke { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000011, pos.y() as u8, pos.x() as u8])?;
            },
            MwEv::CitMoney { cit, money } => {
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
            MwEv::CitIncome { cit, money, income } => {
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
            MwEv::CitMoneyTransact { cit, amount } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000101, *cit])?;
                w.write_all(&amount.to_be_bytes())?;
            },
            MwEv::CitRes { cit, res } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000110, *cit])?;
                w.write_all(&res.to_be_bytes())?;
            },
            MwEv::CitTradeInfo { cit, export, import } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00000111, *cit, *export, *import])?;
            },
            MwEv::Flag { plid, pos } => {
                n_bytes = 4;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00001111, u8::from(*plid), pos.y() as u8, pos.x() as u8])?;
            },
            MwEv::StructureGone { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b00100000, pos.y() as u8, pos.x() as u8])?;
            },
            MwEv::StructureHp { pos, hp } => {
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
            MwEv::BuildNew { pos, kind, pts } => {
                n_bytes = 5;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b01000000 | (*kind as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                w.write_all(&pts.to_be_bytes())?;
            },
            MwEv::Construction { pos, current, rate } => {
                n_bytes = 7;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                w.write_all(&[0b01001111, pos.y() as u8, pos.x() as u8])?;
                w.write_all(&current.to_be_bytes())?;
                w.write_all(&rate.to_be_bytes())?;
            },
            MwEv::RevealStructure { pos, kind } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b01010000 | (*kind as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
            },
            MwEv::RevealItem { pos, item } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b00010000 | (*item as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
            },
            MwEv::TileKind { pos, kind } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let byte0 = 0b01110000 | (*kind as u8 & 0x0F);
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
            },
            MwEv::TileOwner { pos, plid } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let mut byte0 = 0b10000000 | (u8::from(*plid) & 0x0F);
                let mut tilecount = 0;
                for msg in msgs.iter().skip(1) {
                    if tilecount >= 7 {
                        break;
                    }
                    if let MwEv::TileOwner { plid: plid2, pos: _ } = msg {
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
                byte0 |= tilecount << 4;
                w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                for msg in msgs.iter().skip(1).take(tilecount as usize) {
                    let MwEv::TileOwner { pos, plid: _ } = msg else {
                        unreachable!();
                    };
                    w.write_all(&[pos.y() as u8, pos.x() as u8])?;
                }
                n_msgs += tilecount as usize;
            },
            MwEv::Explode { pos } => {
                n_bytes = 3;
                if max_bytes < n_bytes {
                    return Ok((0, 0));
                }
                let mut byte0 = 0b00110000;
                let mut tilecount = 0;
                for msg in msgs.iter().skip(1) {
                    if tilecount >= 15 {
                        break;
                    }
                    if let MwEv::Explode { .. } = msg {
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
                    let MwEv::Explode { pos } = msg else {
                        unreachable!();
                    };
                    w.write_all(&[pos.y() as u8, pos.x() as u8])?;
                }
                n_msgs += tilecount as usize;
            },
            MwEv::DigitCapture { pos, digit, asterisk } => {
                n_bytes = 4;
                let mut tilecount = 0;
                for msg in msgs.iter().skip(1) {
                    if tilecount >= 7 {
                        break;
                    }
                    if let MwEv::DigitCapture { .. } = msg {
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
                    byte0 |= tilecount << 4;
                    w.write_all(&[byte0, pos.y() as u8, pos.x() as u8])?;
                    for msg in msgs.iter().skip(1).take(tilecount as usize) {
                        let MwEv::DigitCapture { pos, .. } = msg else {
                            unreachable!();
                        };
                        w.write_all(&[pos.y() as u8, pos.x() as u8])?;
                    }
                    let mut high = false;
                    let mut digbyte = 0;
                    digbyte |= (*asterisk as u8) << 7;
                    digbyte |= (*digit & 0x07) << 4;
                    for msg in msgs.iter().skip(1).take(tilecount as usize) {
                        let MwEv::DigitCapture { asterisk, digit, .. } = msg else {
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
    fn read<R: std::io::BufRead>(&mut self, r: &mut R, out: &mut Vec<MwEv>) -> Result<usize, Self::Error> {
        let mut byte0 = [0u8];
        match r.read(&mut byte0) {
            Ok(0) => return Ok(0),
            Ok(_) => {},
            Err(e) => return Err(e.into()),
        }
        let byte0 = byte0[0];

        if byte0 == 0b00000000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let plid = PlayerId::from(bytes[0] & 0x0F);
            let subplid = (bytes[0] & 0xF0) >> 4;
            let subplid = if subplid == 15 {
                None
            } else {
                Some(subplid)
            };
            return match bytes[1] {
                0b00000000 => {
                    let mut strlen = [0; 1];
                    r.read_exact(&mut strlen)?;
                    let mut sbuf = vec![0; strlen[0] as usize];
                    r.read_exact(&mut sbuf)?;
                    let name = String::from_utf8(sbuf)?;
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Joined { name } });
                    Ok(1)
                }
                0b00000001 => {
                    let mut dur = [0; 1];
                    r.read_exact(&mut dur)?;
                    let duration = MwDur(dur[0]);
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::NetRttInfo { duration } });
                    Ok(1)
                }
                0b00000010 => {
                    let mut dur = [0; 1];
                    r.read_exact(&mut dur)?;
                    let duration = MwDur(dur[0]);
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Timeout { duration } });
                    Ok(1)
                }
                0b00000011 => {
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::TimeoutFinished });
                    Ok(1)
                }
                0b00000100 => {
                    let mut data = [0; 3];
                    r.read_exact(&mut data)?;
                    let mut pos = Pos::default();
                    pos.set_y(data[0] as i8);
                    pos.set_x(data[1] as i8);
                    let killer = PlayerId::from(data[2]);
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Exploded { pos, killer } });
                    Ok(1)
                }
                0b00000101 => {
                    let mut lives = [0; 1];
                    r.read_exact(&mut lives)?;
                    let lives = lives[0];
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::LivesRemain { lives } });
                    Ok(1)
                }
                0b00000110 => {
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Protected });
                    Ok(1)
                }
                0b00000111 => {
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Unprotected });
                    Ok(1)
                }
                0b00001000 => {
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Eliminated });
                    Ok(1)
                }
                0b00001001 => {
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Surrendered });
                    Ok(1)
                }
                0b00001010 => {
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Disconnected });
                    Ok(1)
                }
                0b00001011 => {
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::Kicked });
                    Ok(1)
                }
                0b00001100 => {
                    let mut id = [0; 1];
                    r.read_exact(&mut id)?;
                    let id = id[0];
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::VoteNo { id } });
                    Ok(1)
                }
                0b00001101 => {
                    let mut id = [0; 1];
                    r.read_exact(&mut id)?;
                    let id = id[0];
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::VoteYes { id } });
                    Ok(1)
                }
                0b00001110 => {
                    let mut id = [0; 1];
                    r.read_exact(&mut id)?;
                    let id = id[0];
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::VoteFail { id } });
                    Ok(1)
                }
                0b00001111 => {
                    let mut id = [0; 1];
                    r.read_exact(&mut id)?;
                    let id = id[0];
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::VotePass { id } });
                    Ok(1)
                }
                0b00010000 => {
                    let mut strlen = [0; 1];
                    r.read_exact(&mut strlen)?;
                    let mut sbuf = vec![0; strlen[0] as usize];
                    r.read_exact(&mut sbuf)?;
                    let text = String::from_utf8(sbuf)?;
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::ChatAll { text } });
                    Ok(1)
                }
                0b00010001 => {
                    let mut strlen = [0; 1];
                    r.read_exact(&mut strlen)?;
                    let mut sbuf = vec![0; strlen[0] as usize];
                    r.read_exact(&mut sbuf)?;
                    let text = String::from_utf8(sbuf)?;
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::ChatFriendly { text } });
                    Ok(1)
                }
                0b00010010 => {
                    let mut data = [0; 2];
                    r.read_exact(&mut data)?;
                    let secs = u16::from_be_bytes(data);
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::MatchTimeRemain { secs } });
                    Ok(1)
                }
                0b00010011 => {
                    let mut data = [0; 2];
                    r.read_exact(&mut data)?;
                    let id = data[0];
                    let mut sbuf = vec![0; data[1] as usize];
                    r.read_exact(&mut sbuf)?;
                    let l10nkey = String::from_utf8(sbuf)?;
                    out.push(MwEv::Player { plid, subplid, ev: PlayerEv::VoteNew { id, l10nkey } });
                    Ok(1)
                }
                _ => Err(MsgBinReadError::UnknownPlayerOp(bytes[1])),
            };
        }
        if byte0 == 0b00000001 {
            out.push(MwEv::Tremor);
            return Ok(1);
        }
        if byte0 & 0b11111110 == 0b00000010 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            if byte0 & 1 == 0 {
                out.push(MwEv::Smoke { pos });
            } else {
                out.push(MwEv::Unsmoke { pos });
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
                out.push(MwEv::CitIncome { cit, money, income });
            } else {
                out.push(MwEv::CitMoney { cit, money });
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
            out.push(MwEv::CitMoneyTransact { cit, amount });
            return Ok(1);
        }
        if byte0 == 0b00000110 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let cit = bytes[0];
            let res = u16::from_be_bytes([
                bytes[1], bytes[2]
            ]);
            out.push(MwEv::CitRes { cit, res });
            return Ok(1);
        }
        if byte0 == 0b00000111 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let cit = bytes[0];
            let export = bytes[1];
            let import = bytes[2];
            out.push(MwEv::CitTradeInfo { cit, export, import });
            return Ok(1);
        }
        if byte0 == 0b00001110 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[1] as i8);
            pos.set_x(bytes[2] as i8);
            out.push(MwEv::Debug(bytes[0], pos));
            return Ok(1);
        }
        if byte0 == 0b00001111 {
            let mut bytes = [0; 3];
            r.read_exact(&mut bytes)?;
            let plid = PlayerId::from(bytes[0]);
            let mut pos = Pos::default();
            pos.set_y(bytes[1] as i8);
            pos.set_x(bytes[2] as i8);
            out.push(MwEv::Flag { plid, pos });
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
                out.push(MwEv::StructureHp { pos, hp });
            } else {
                out.push(MwEv::StructureGone { pos });
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
            out.push(MwEv::Construction { pos, current, rate });
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
            let Some(kind) = StructureKind::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(MwEv::BuildNew { pos, kind, pts });
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
            let Some(kind) = StructureKind::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(MwEv::RevealStructure { pos, kind });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b00010000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let kind_byte = byte0 & 0x0F;
            let Some(item) = ItemKind::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(MwEv::RevealItem { pos, item });
            return Ok(1);
        }
        if byte0 & 0b11110000 == 0b01110000 {
            let mut bytes = [0; 2];
            r.read_exact(&mut bytes)?;
            let mut pos = Pos::default();
            pos.set_y(bytes[0] as i8);
            pos.set_x(bytes[1] as i8);
            let kind_byte = byte0 & 0x0F;
            let Some(kind) = TileKind::from_u8(kind_byte) else {
                return Err(MsgBinReadError::InvalidValue(kind_byte as u32));
            };
            out.push(MwEv::TileKind { pos, kind });
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
            out.push(MwEv::DigitCapture { pos, digit, asterisk });
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
                out.push(MwEv::Explode { pos });
            }
            return Ok(n_tiles);
        }
        if byte0 & 0b10001111 == 0b10000000 {
            let mut bytes = [0; 2 * 16 + 8];
            let n_tiles = (((byte0 & 0b01110000) >> 4) + 1) as usize;
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
                out.push(MwEv::DigitCapture { pos, digit, asterisk });
            }
            return Ok(n_tiles);
        }
        if byte0 & 0b10000000 == 0b10000000 {
            let mut bytes = [0; 2 * 8];
            let plid = PlayerId::from(byte0 & 0b00001111);
            let n_tiles = (((byte0 & 0b01110000) >> 4) + 1) as usize;
            r.read_exact(&mut bytes[..(2 * n_tiles)])?;
            for i in 0..n_tiles {
                let mut pos = Pos::default();
                pos.set_y(bytes[i * 2 + 0] as i8);
                pos.set_x(bytes[i * 2 + 1] as i8);
                out.push(MwEv::TileOwner { pos, plid });
            }
            return Ok(n_tiles);
        }

        Err(MsgBinReadError::UnknownOp(byte0))
    }
}
