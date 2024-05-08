use std::io::{BufReader, BufWriter, BufRead, Write};

use mw_dataformat::msg::asm::MsgAsmRead;
use mw_dataformat::msg::bin::MsgBinWrite;
use mw_dataformat::msg::{MsgReader, MsgWriter};

use crate::prelude::*;
use crate::{CommonArgs, AsmArgs};

pub fn main(common: &CommonArgs, args: &AsmArgs) -> AnyResult<()> {
    match (&common.input, &common.output) {
        (None, Some(out_path)) => {
            let out_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(true)
                .open(out_path)
                .context("Cannot open output file!")?;
            let bufr = BufReader::new(std::io::stdin());
            let bufw = BufWriter::new(out_file);
            asm(bufr, bufw, args)?;
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
            asm(bufr, bufw, args)?;
        }
        (_, None) => {
            bail!("Output filename must be specified!");
        }
    }
    Ok(())
}

fn asm<R: BufRead, W: Write>(mut reader: R, mut writer: W, args: &AsmArgs) -> AnyResult<()> {
    if args.unframed {
        let mut msgs = vec![];
        let mut r_asm = MsgAsmRead::new();
        let mut w_bin = MsgBinWrite::new();
        r_asm.read_all(&mut reader, &mut msgs)
            .context("Failed to decode ASM message stream")?;
        w_bin.write_all(&mut writer, &msgs)
            .context("Failed to encode binary message stream")?;
    } else {
        todo!()
    }
    Ok(())
}
