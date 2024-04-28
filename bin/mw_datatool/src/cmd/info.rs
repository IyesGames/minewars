use mw_common::phoneme::{lang, render_str};
use mw_dataformat::read::MwFileReader;

use crate::prelude::*;
use crate::{CommonArgs, InfoArgs};

pub fn main(common: &CommonArgs, args: &InfoArgs) -> AnyResult<()> {
    let file = if let Some(in_path) = &common.input {
        std::fs::OpenOptions::new()
            .read(true)
            .open(in_path)
            .context("Cannot read input file!")?
    } else {
        bail!("Input filename must be specified!");
    };
    let mut buf = Vec::new();

    let mut mfr = MwFileReader::new(file, &mut buf)
        .context("Failed to load input file as a MineWars format file!")?;

    if !args.ignore_checksums {
        mfr.verify_checksums()
            .context("Checksum verification failed!")?;
    }

    eprintln!("File Header:");
    let checksum_header = mfr.file_header().checksum_header;
    eprintln!("Header checksum: {:016x}", checksum_header);
    let checksum_is = mfr.file_header().checksum_is;
    eprintln!("ISData checksum: {:016x}", checksum_is);
    let checksum_framedata = mfr.file_header().checksum_framedata;
    eprintln!("Frames checksum: {:016x}", checksum_framedata);
    eprintln!("FrameData is compressed?: {}", mfr.is_framedata_compressed());
    eprintln!("FrameData length (compressed): {}", mfr.file_header().len_framedata_compressed());
    eprintln!("FrameData length (raw):        {}", mfr.file_header().len_framedata_raw());

    let (_, mut isr) = mfr.read_is()
        .context("Cannot read IS")?;

    eprintln!();
    eprintln!("IS Header:");
    eprintln!("Map Topology: {:?}", isr.map_topology());
    eprintln!("Map Size: {}", isr.map_size());
    eprintln!("Number of map regions: {}", isr.n_regions());
    eprintln!("MapData is compressed?: {}", isr.is_mapdata_compressed());
    eprintln!("MapData length (compressed): {}", isr.header().len_mapdata_compressed());
    eprintln!("MapData length (raw):        {}", isr.header().len_mapdata_raw());
    eprintln!("Number of players: {}", isr.n_players());
    eprintln!("Player names anonymized?: {}", isr.header().len_playerdata() == 0);
    eprintln!("PlayerData length: {}", isr.header().len_playerdata());
    eprintln!("RulesData length: {}", isr.header().len_rules());

    eprintln!();
    eprintln!("Player Names:");
    for (i, name) in isr.read_players()?.enumerate() {
        eprintln!("{}: {:?}", i, name);
    }

    eprintln!();
    eprintln!("Cits:");
    let cit_pos = isr.read_cits_pos()?.to_owned();
    let iter_cit_names = isr.read_cits_names()?;
    for (i, (pos, name)) in cit_pos.iter().cloned().zip(iter_cit_names).enumerate() {
        eprintln!("{}: Y:{},X:{} {:?}", i, pos.y(), pos.x(), render_str::<lang::EN>(name));
    }

    Ok(())
}
