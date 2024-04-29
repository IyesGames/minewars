//! "Assembly Language" for Game Event Messages
//!
//! The idea is to help with debugging and development of the MineWars
//! data stream by having a human-readable representation of the binary data.

use mw_common::{grid::Pos, plid::PlayerId};
use thiserror::Error;

use crate::msg::*;

#[derive(Debug, Error)]
pub enum MsgAsmWriteError {
    #[error("Formatter: {0}")]
    Fmt(#[from] std::fmt::Error),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum MsgAsmReadError {
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unknown Instruction: {0}")]
    UnknownOp(String),
    #[error("Invalid Operand: {0}")]
    BadArg(String),
    #[error("Expected more operands!")]
    NotEnoughArgs,
    #[error("Too many operands!")]
    TooManyArgs,
}

pub struct MsgAsmRead {
    buf: String,
}
pub struct MsgAsmWrite {
    buf: String,
}

impl MsgAsmRead {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
        }
    }
}

impl MsgAsmWrite {
    pub fn new() -> Self {
        Self {
            buf: String::new(),
        }
    }
}

impl MsgWriter for MsgAsmWrite {
    type Error = MsgAsmWriteError;

    fn write<W: std::io::Write>(&mut self, w: &mut W, msgs: &[Msg], max_bytes: usize) -> Result<(usize, usize), Self::Error> {
        use std::fmt::Write;
        let Some(first) = msgs.get(0) else {
            return Ok((0, 0));
        };

        self.buf.clear();
        let mut n_msgs = 1;

        match first {
            Msg::Player { plid, subplid, status } => {
                if let Some(subplid) = subplid {
                    write!(&mut self.buf, "PLAYER {}/{}", u8::from(*plid), subplid)?;
                } else {
                    write!(&mut self.buf, "PLAYER {}", u8::from(*plid))?;
                }
                match status {
                    MsgPlayer::Joined => write!(&mut self.buf, " JOIN")?,
                    MsgPlayer::NetRttInfo { duration } => write!(&mut self.buf, " RTT {}", duration.as_millis())?,
                    MsgPlayer::Timeout { duration } => write!(&mut self.buf, " TIMEOUT {}", duration.as_millis())?,
                    MsgPlayer::TimeoutFinished => write!(&mut self.buf, " RESUME")?,
                    MsgPlayer::Exploded { pos, killer } => write!(&mut self.buf, " EXPLODE {},{} {}", pos.y(), pos.x(), u8::from(*killer))?,
                    MsgPlayer::LivesRemain { lives } => write!(&mut self.buf, " LIVES {}", *lives)?,
                    MsgPlayer::Protected => write!(&mut self.buf, " PROTECT")?,
                    MsgPlayer::Unprotected => write!(&mut self.buf, " UNPROTECT")?,
                    MsgPlayer::Eliminated => write!(&mut self.buf, " ELIMINATE")?,
                    MsgPlayer::Surrendered => write!(&mut self.buf, " SURRENDER")?,
                    MsgPlayer::Disconnected => write!(&mut self.buf, " LEAVE")?,
                    MsgPlayer::Kicked => write!(&mut self.buf, " KICK")?,
                    MsgPlayer::MatchTimeRemain { secs } => write!(&mut self.buf, " TIMELIMIT {}", *secs)?,
                }
                writeln!(&mut self.buf, "")?;
            },
            Msg::DigitCapture { pos, digit, asterisk } => {
                write!(&mut self.buf, "DIGITS {}{}/{},{}", digit, if *asterisk { "*" } else { "" }, pos.y(), pos.x())?;
                for (i, msg) in msgs[1..].iter().enumerate() {
                    if i >= 15 {
                        break;
                    }
                    if let Msg::DigitCapture { pos, digit, asterisk } = msg {
                        write!(&mut self.buf, " {}{}/{},{}", digit, if *asterisk { "*" } else { "" }, pos.y(), pos.x())?;
                        n_msgs += 1;
                    } else {
                        break;
                    }
                }
                writeln!(&mut self.buf, "")?;
            },
            Msg::TileOwner { pos, plid } => {
                write!(&mut self.buf, "OWNER {} {},{}", u8::from(*plid), pos.y(), pos.x())?;
                for (i, msg) in msgs[1..].iter().enumerate() {
                    if i >= 7 {
                        break;
                    }
                    if let Msg::TileOwner { pos, plid: plid_new } = msg {
                        if plid_new != plid {
                            break;
                        }
                        write!(&mut self.buf, " {},{}", pos.y(), pos.x())?;
                        n_msgs += 1;
                    } else {
                        break;
                    }
                }
                writeln!(&mut self.buf, "")?;
            },
            Msg::CitRes { cit, res } => {
                writeln!(&mut self.buf, "CITRES {} {}", cit, res)?;
            },
            Msg::CitMoneyTransact { cit, amount } => {
                writeln!(&mut self.buf, "CITTRANS {} {}", cit, amount)?;
            },
            Msg::CitMoney { cit, money } => {
                writeln!(&mut self.buf, "CITMONEY {} {}", cit, money)?;
            },
            Msg::CitIncome { cit, money, income } => {
                writeln!(&mut self.buf, "CITINCOME {} {} {}", cit, money, income)?;
            },
            Msg::CitTradeInfo { cit, export, import } => {
                writeln!(&mut self.buf, "CITTRADE {} {} {}", cit, export, import)?;
            },
            Msg::RevealStructure { pos, kind } => {
                writeln!(&mut self.buf, "STRUCT {},{} {}", pos.y(), pos.x(), match kind {
                    MsgStructureKind::Road => "road",
                    MsgStructureKind::Bridge => "bridge",
                    MsgStructureKind::Wall => "wall",
                    MsgStructureKind::Tower => "tower",
                })?;
            },
            Msg::StructureGone { pos } => {
                writeln!(&mut self.buf, "DECONSTRUCT {},{}", pos.y(), pos.x())?;
            },
            Msg::StructureHp { pos, hp } => {
                writeln!(&mut self.buf, "STRUCTHP {},{} {}", pos.y(), pos.x(), hp)?;
            },
            Msg::BuildNew { pos, kind, pts } => {
                writeln!(&mut self.buf, "BUILDNEW {},{} {} {}", pos.y(), pos.x(), match kind {
                    MsgStructureKind::Road => "road",
                    MsgStructureKind::Bridge => "bridge",
                    MsgStructureKind::Wall => "wall",
                    MsgStructureKind::Tower => "tower",
                }, pts)?;
            },
            Msg::Construction { pos, current, rate } => {
                writeln!(&mut self.buf, "BUILD {},{} {} {}", pos.y(), pos.x(), current, rate)?;
            },
            Msg::RevealItem { pos, item }  => {
                writeln!(&mut self.buf, "ITEM {},{} {}", pos.y(), pos.x(), match item {
                    MsgItem::None => "none",
                    MsgItem::Decoy => "decoy",
                    MsgItem::Mine => "mine",
                    MsgItem::Trap => "trap",
                })?;
            },
            Msg::Explode { pos } => {
                write!(&mut self.buf, "EXPLODE {},{}", pos.y(), pos.x())?;
                for (i, msg) in msgs[1..].iter().enumerate() {
                    if i >= 15 {
                        break;
                    }
                    if let Msg::Explode { pos } = msg {
                        write!(&mut self.buf, " {},{}", pos.y(), pos.x())?;
                        n_msgs += 1;
                    } else {
                        break;
                    }
                }
                writeln!(&mut self.buf, "")?;
            },
            Msg::Smoke { pos } => {
                writeln!(&mut self.buf, "SMOKE {},{}", pos.y(), pos.x())?;
            },
            Msg::Unsmoke { pos } => {
                writeln!(&mut self.buf, "UNSMOKE {},{}", pos.y(), pos.x())?;
            },
            Msg::Tremor => {
                writeln!(&mut self.buf, "SHAKE")?;
            },
            Msg::Nop => {
                writeln!(&mut self.buf, "NOP")?;
            },
            Msg::Flag { plid, pos } => {
                writeln!(&mut self.buf, "FLAG {} {},{}", u8::from(*plid), pos.y(), pos.x())?;
            },
            Msg::TileKind { pos, kind } => {
                writeln!(&mut self.buf, "TILE {},{} {}", pos.y(), pos.x(), match kind {
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

        if self.buf.len() > max_bytes {
            Ok((0, 0))
        } else {
            w.write_all(self.buf.as_bytes())?;
            Ok((n_msgs, self.buf.len()))
        }
    }
}
impl MsgReader for MsgAsmRead {
    type Error = MsgAsmReadError;

    fn read<R: std::io::BufRead>(&mut self, r: &mut R, out: &mut Vec<Msg>) -> Result<usize, Self::Error> {
        self.buf.clear();
        r.read_line(&mut self.buf)?;
        // get the part before any comment and trim whitespace
        let Some(line) = self.buf.split(';').next().map(|s| s.trim()) else {
            return Ok(0);
        };
        if line.is_empty() {
            return Ok(0);
        }
        let mut components = line.split_ascii_whitespace();
        let Some(iname) = components.next() else {
            return Ok(0);
        };
        if line.is_empty() {
            return Ok(0);
        }

        match iname.to_ascii_uppercase().as_str() {
            "PLAYER" => {
                let Some(arg_plid) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_status) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let (arg_plid, arg_subplid) = arg_plid.split_once('/')
                    .unwrap_or((arg_plid, ""));
                let Ok(plid) = arg_plid.parse::<u8>() else {
                    return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                };
                let plid = PlayerId::from(plid);
                let subplid = if arg_subplid.is_empty() {
                    None
                } else {
                    let Ok(subplid) = arg_subplid.parse::<u8>() else {
                        return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                    };
                    Some(subplid)
                };
                let status = match arg_status.to_ascii_uppercase().as_str() {
                    "JOIN" => MsgPlayer::Joined,
                    "RTT" => {
                        let Some(arg_duration) = components.next() else {
                            return Err(MsgAsmReadError::NotEnoughArgs);
                        };
                        let Ok(duration) = arg_duration.parse::<u16>() else {
                            return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                        };
                        MsgPlayer::NetRttInfo { duration: MwDur::from_millis_lossy(duration) }
                    },
                    "TIMEOUT" => {
                        let Some(arg_duration) = components.next() else {
                            return Err(MsgAsmReadError::NotEnoughArgs);
                        };
                        let Ok(duration) = arg_duration.parse::<u16>() else {
                            return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                        };
                        MsgPlayer::Timeout { duration: MwDur::from_millis_lossy(duration) }
                    },
                    "RESUME" => MsgPlayer::TimeoutFinished,
                    "EXPLODE" => {
                        let Some(arg_pos) = components.next() else {
                            return Err(MsgAsmReadError::NotEnoughArgs);
                        };
                        let Some(arg_plid) = components.next() else {
                            return Err(MsgAsmReadError::NotEnoughArgs);
                        };
                        let pos = parse_pos(arg_pos)?;
                        let Ok(plid) = arg_plid.parse::<u8>() else {
                            return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                        };
                        let plid = PlayerId::from(plid);
                        MsgPlayer::Exploded { pos, killer: plid }
                    },
                    "LIVES" => {
                        let Some(arg_lives) = components.next() else {
                            return Err(MsgAsmReadError::NotEnoughArgs);
                        };
                        let Ok(lives) = arg_lives.parse::<u8>() else {
                            return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                        };
                        MsgPlayer::LivesRemain { lives }
                    },
                    "PROTECT" => MsgPlayer::Protected,
                    "UNPROTECT" => MsgPlayer::Unprotected,
                    "ELIMINATE" => MsgPlayer::Eliminated,
                    "SURRENDER" => MsgPlayer::Surrendered,
                    "LEAVE" => MsgPlayer::Disconnected,
                    "KICK" => MsgPlayer::Kicked,
                    "TIMELIMIT" => {
                        let Some(arg_secs) = components.next() else {
                            return Err(MsgAsmReadError::NotEnoughArgs);
                        };
                        let Ok(secs) = arg_secs.parse::<u16>() else {
                            return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                        };
                        MsgPlayer::MatchTimeRemain { secs }
                    },
                    other => {
                        return Err(MsgAsmReadError::BadArg(other.to_owned()));
                    }
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                out.push(Msg::Player {
                    plid, subplid, status,
                });
                Ok(1)
            }
            "TILE" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_kind) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
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
                        return Err(MsgAsmReadError::BadArg(other.to_owned()));
                    }
                };
                out.push(Msg::TileKind {
                    pos, kind,
                });
                Ok(1)
            }
            "DIGITS" => {
                let mut n = 0;
                for arg in components {
                    let Some((mut arg_digit, arg_pos)) = arg.split_once('/') else {
                        return Err(MsgAsmReadError::BadArg(arg.to_owned()));
                    };
                    let asterisk = if let Some(s) = arg_digit.strip_suffix('*') {
                        arg_digit = s;
                        true
                    } else {
                        false
                    };
                    let Ok(digit) = arg_digit.parse() else {
                        return Err(MsgAsmReadError::BadArg(arg_digit.to_owned()));
                    };
                    let pos = parse_pos(arg_pos)?;
                    out.push(Msg::DigitCapture {
                        digit, pos, asterisk,
                    });
                    n += 1;
                }
                if n == 0 {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                }
                Ok(n)
            }
            "OWNER" => {
                let Some(arg_plid) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Ok(plid) = arg_plid.parse::<u8>() else {
                    return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                };
                if plid > 7 {
                    return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                }
                let plid = PlayerId::from(plid);
                let mut n = 0;
                for arg in components {
                    let pos = parse_pos(arg)?;
                    out.push(Msg::TileOwner {
                        plid, pos,
                    });
                    n += 1;
                }
                if n == 0 {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                }
                Ok(n)
            }
            "FLAG" => {
                let Some(arg_plid) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let Ok(plid) = arg_plid.parse::<u8>() else {
                    return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                };
                if plid > 7 {
                    return Err(MsgAsmReadError::BadArg(arg_plid.to_owned()));
                }
                let plid = PlayerId::from(plid);
                let pos = parse_pos(arg_pos)?;
                out.push(Msg::Flag {
                    pos, plid,
                });
                Ok(1)
            }
            "CITRES" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_res) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_cit.to_owned()));
                };
                let Ok(res) = arg_res.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_res.to_owned()));
                };
                out.push(Msg::CitRes {
                    cit, res,
                });
                Ok(1)
            }
            "CITTRANS" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_amount) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_cit.to_owned()));
                };
                let Ok(amount) = arg_amount.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_amount.to_owned()));
                };
                out.push(Msg::CitMoneyTransact {
                    cit, amount,
                });
                Ok(1)
            }
            "CITMONEY" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_money) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_cit.to_owned()));
                };
                let Ok(money) = arg_money.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_money.to_owned()));
                };
                out.push(Msg::CitMoney {
                    cit, money,
                });
                Ok(1)
            }
            "CITINCOME" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_money) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_income) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_cit.to_owned()));
                };
                let Ok(money) = arg_money.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_money.to_owned()));
                };
                let Ok(income) = arg_income.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_income.to_owned()));
                };
                out.push(Msg::CitIncome {
                    cit, money, income,
                });
                Ok(1)
            }
            "CITTRADE" => {
                let Some(arg_cit) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_export) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_import) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let Ok(cit) = arg_cit.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_cit.to_owned()));
                };
                let Ok(export) = arg_export.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_export.to_owned()));
                };
                let Ok(import) = arg_import.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_import.to_owned()));
                };
                out.push(Msg::CitTradeInfo {
                    cit, export, import,
                });
                Ok(1)
            }
            "STRUCT" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_kind) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let kind = match arg_kind.to_ascii_uppercase().as_str() {
                    "ROAD" => MsgStructureKind::Road,
                    "BRIDGE" => MsgStructureKind::Bridge,
                    "WALL" => MsgStructureKind::Wall,
                    "TOWER" => MsgStructureKind::Tower,
                    other => {
                        return Err(MsgAsmReadError::BadArg(other.to_owned()));
                    }
                };
                out.push(Msg::RevealStructure {
                    pos, kind,
                });
                Ok(1)
            }
            "DECONSTRUCT" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                out.push(Msg::StructureGone {
                    pos
                });
                Ok(1)
            }
            "STRUCTHP" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_hp) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let Ok(hp) = arg_hp.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_hp.to_owned()));
                };
                out.push(Msg::StructureHp {
                    pos, hp,
                });
                Ok(1)
            }
            "BUILDNEW" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_kind) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_pts) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let kind = match arg_kind.to_ascii_uppercase().as_str() {
                    "ROAD" => MsgStructureKind::Road,
                    "BRIDGE" => MsgStructureKind::Bridge,
                    "WALL" => MsgStructureKind::Wall,
                    "TOWER" => MsgStructureKind::Tower,
                    other => {
                        return Err(MsgAsmReadError::BadArg(other.to_owned()));
                    }
                };
                let Ok(pts) = arg_pts.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_pts.to_owned()));
                };
                out.push(Msg::BuildNew {
                    pos, kind, pts,
                });
                Ok(1)
            }
            "BUILD" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_current) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_rate) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let Ok(current) = arg_current.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_current.to_owned()));
                };
                let Ok(rate) = arg_rate.parse() else {
                    return Err(MsgAsmReadError::BadArg(arg_current.to_owned()));
                };
                out.push(Msg::Construction {
                    pos, current, rate
                });
                Ok(1)
            }
            "ITEM" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                let Some(arg_item) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                let item = match arg_item.to_ascii_uppercase().as_str() {
                    "NONE" => MsgItem::None,
                    "DECOY" => MsgItem::Decoy,
                    "MINE" => MsgItem::Mine,
                    "TRAP" => MsgItem::Trap,
                    other => {
                        return Err(MsgAsmReadError::BadArg(other.to_owned()));
                    }
                };
                out.push(Msg::RevealItem {
                    pos, item
                });
                Ok(1)
            }
            "EXPLODE" => {
                let mut n = 0;
                for arg in components {
                    let pos = parse_pos(arg)?;
                    out.push(Msg::Explode {
                        pos,
                    });
                    n += 1;
                }
                if n == 0 {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                }
                Ok(n)
            }
            "SMOKE" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                out.push(Msg::Smoke {
                    pos
                });
                Ok(1)
            }
            "UNSMOKE" => {
                let Some(arg_pos) = components.next() else {
                    return Err(MsgAsmReadError::NotEnoughArgs);
                };
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                let pos = parse_pos(arg_pos)?;
                out.push(Msg::Unsmoke {
                    pos
                });
                Ok(1)
            }
            "SHAKE" => {
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                out.push(Msg::Tremor);
                Ok(1)
            }
            "NOP" => {
                if components.next().is_some() {
                    return Err(MsgAsmReadError::TooManyArgs);
                }
                out.push(Msg::Nop);
                Ok(1)
            }
            other => {
                Err(MsgAsmReadError::UnknownOp(other.to_owned()))
            }
        }
    }
}

fn parse_pos(s: &str) -> Result<Pos, MsgAsmReadError> {
    // error we return if anything goes wrong
    let err = Err(MsgAsmReadError::BadArg(s.to_owned()));

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
