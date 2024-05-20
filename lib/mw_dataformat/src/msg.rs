//! Working with individual Game Update Messages

use mw_common::prelude::*;

pub mod asm;
pub mod bin;

pub trait MsgWriter {
    type Error: std::error::Error;
    fn write<W: std::io::Write>(&mut self, w: &mut W, msgs: &[MwEv], max_bytes: usize) -> Result<(usize, usize), Self::Error>;
    fn write_many<W: std::io::Write>(&mut self, w: &mut W, mut msgs: &[MwEv], max_bytes: usize) -> Result<(usize, usize), Self::Error> {
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
    fn write_all<W: std::io::Write>(&mut self, w: &mut W, mut msgs: &[MwEv]) -> Result<(), Self::Error> {
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
    fn read<R: std::io::BufRead>(&mut self, r: &mut R, out: &mut Vec<MwEv>) -> Result<usize, Self::Error>;
    fn read_all<R: std::io::BufRead>(&mut self, r: &mut R, out: &mut Vec<MwEv>) -> Result<usize, Self::Error> {
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
