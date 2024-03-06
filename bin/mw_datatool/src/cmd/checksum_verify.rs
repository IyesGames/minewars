use mw_dataformat::read::MwFileReader;

use crate::prelude::*;
use crate::{CommonArgs, ChecksumVerifyArgs};

pub fn main(common: &CommonArgs, _args: &ChecksumVerifyArgs) -> AnyResult<()> {
    let file = if let Some(in_path) = &common.input {
        std::fs::OpenOptions::new()
            .read(true)
            .open(in_path)
            .context("Cannot open input file!")?
    } else {
        bail!("Input filename must be specified!");
    };
    let mut buf = Vec::new();

    let mut mfr = MwFileReader::new(file, &mut buf)
        .context("Failed to load input file as a MineWars format file!")?;

    let mut all_good = true;

    let checksum_expected = mfr.file_header().checksum_header;
    match mfr.compute_new_checksum_header() {
        Ok(checksum) => {
            if checksum == checksum_expected {
                eprintln!("Header checksum: {:016x} (ok!)", checksum);
            } else {
                eprintln!("Header checksum: {:016x} (BAD! Expected: {:016x})", checksum, checksum_expected);
                all_good = false;
            }
        }
        Err(e) => {
            eprintln!("Header checksum: Expected: {:016x} Cannot verify! Error: {:#}", checksum_expected, e);
            all_good = false;
        }
    }
    let checksum_expected = mfr.file_header().checksum_is;
    match mfr.compute_new_checksum_isdata() {
        Ok(checksum) => {
            if checksum == checksum_expected {
                eprintln!("ISData checksum: {:016x} (ok!)", checksum);
            } else {
                eprintln!("ISData checksum: {:016x} (BAD! Expected: {:016x})", checksum, checksum_expected);
                all_good = false;
            }
        }
        Err(e) => {
            eprintln!("ISData checksum: Expected: {:016x} Cannot verify! Error: {:#}", checksum_expected, e);
            all_good = false;
        }
    }
    let checksum_expected = mfr.file_header().checksum_framedata;
    match mfr.compute_new_checksum_framedata() {
        Ok(checksum) => {
            if checksum == checksum_expected {
                eprintln!("Frames checksum: {:016x} (ok!)", checksum);
            } else {
                eprintln!("Frames checksum: {:016x} (BAD! Expected: {:016x})", checksum, checksum_expected);
                all_good = false;
            }
        }
        Err(e) => {
            eprintln!("Frames checksum: Expected: {:016x} Cannot verify! Error: {:#}", checksum_expected, e);
            all_good = false;
        }
    }

    if all_good {
        Ok(())
    } else {
        bail!("Checksum verification failed!");
    }
}
