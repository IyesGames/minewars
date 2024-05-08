use std::io::{BufReader, BufWriter, BufRead, Write};

use mw_dataformat::msg::asm::MsgAsmWrite;
use mw_dataformat::msg::bin::MsgBinRead;
use mw_dataformat::msg::{MsgReader, MsgWriter};

use crate::prelude::*;
use crate::{CommonArgs, DisasmArgs};

pub fn main(common: &CommonArgs, args: &DisasmArgs) -> AnyResult<()> {
    match (&common.input, &common.output) {
        (Some(in_path), None) => {
            let in_file = std::fs::OpenOptions::new()
                .read(true)
                .open(in_path)
                .context("Cannot open input file!")?;
            let bufr = BufReader::new(in_file);
            disasm(bufr, std::io::stdout(), args)?;
        }
        (Some(in_path), Some(out_path)) => {
            let in_file = std::fs::OpenOptions::new()
                .read(true)
                .open(in_path)
                .context("Cannot open input file!")?;
            let out_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(out_path)
                .context("Cannot open output file!")?;
            let bufr = BufReader::new(in_file);
            let bufw = BufWriter::new(out_file);
            disasm(bufr, bufw, args)?;
        }
        (None, _) => {
            bail!("Input filename must be specified!");
        }
    }
    Ok(())
}

fn disasm<R: BufRead, W: Write>(mut reader: R, mut writer: W, args: &DisasmArgs) -> AnyResult<()> {
    if args.unframed {
        let mut msgs = vec![];
        let mut r_bin = MsgBinRead::new();
        let mut w_asm = MsgAsmWrite::new();
        loop {
            msgs.clear();
            r_bin.read(&mut reader, &mut msgs)
                .context("Failed to decode binary messages")?;
            if msgs.is_empty() {
                break;
            }
            w_asm.write_all(&mut writer, &msgs)
                .context("Failed to encode ASM messages")?;
        }
    } else {
        todo!()
    }
    Ok(())
}

