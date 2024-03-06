use std::io::{Read, Seek, SeekFrom, Write};

use mw_dataformat::read::MwFileReader;

use crate::prelude::*;
use crate::{CommonArgs, ChecksumFixArgs};

pub fn main(common: &CommonArgs, _args: &ChecksumFixArgs) -> AnyResult<()> {
    match (&common.input, &common.output) {
        (Some(in_path), None) => {
            let file = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(false)
                .create(false)
                .open(in_path)
                .context("Cannot open input file!")?;
            let (file, headerdata) = gen_new_headerdata(file)?;
            write_new_headerdata(file, &headerdata)?;
        }
        (Some(in_path), Some(out_path)) => {
            std::fs::copy(in_path, out_path)
                .context("Failed to copy data from input to output file!")?;
            let in_file = std::fs::OpenOptions::new()
                .read(true)
                .open(in_path)
                .context("Cannot open input file!")?;
            let out_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(false)
                .create(false)
                .open(out_path)
                .context("Cannot open output file!")?;
            let (_, headerdata) = gen_new_headerdata(in_file)?;
            write_new_headerdata(out_file, &headerdata)?;
        }
        (None, _) => {
            bail!("Input filename must be specified!");
        }
    }

    Ok(())
}

fn gen_new_headerdata<R: Read + Seek>(reader: R) -> AnyResult<(R, Vec<u8>)> {
    let mut buf = Vec::new();

    let mut mfr = MwFileReader::new(reader, &mut buf)
        .context("Failed to load input file as a MineWars format file!")?;

    mfr.compute_and_force_update_checksums()?;

    let mut buf = vec![];
    mfr.file_header().serialize(&mut buf);

    Ok((mfr.into_inner(), buf))
}

fn write_new_headerdata<W: Write + Seek>(mut writer: W, headerdata: &[u8]) -> AnyResult<W> {
    writer.seek(SeekFrom::Start(0))?;
    writer.write_all(&headerdata)?;
    Ok(writer)
}
