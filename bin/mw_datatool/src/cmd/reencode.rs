use std::io::{Read, Seek, Write};

use mw_common::game::MapGenTileData;
use mw_common::grid::MapDataTopo;
use mw_dataformat::read::MwFileReader;
use mw_dataformat::write::MwFileBuilder;

use crate::prelude::*;
use crate::{CommonArgs, ReencodeArgs};

pub fn main(common: &CommonArgs, args: &ReencodeArgs) -> AnyResult<()> {
    match (&common.input, &common.output) {
        (Some(in_path), None) => {
            let file_in_mem = std::fs::read(in_path)
                .context("Cannot read input file!")?;
            let file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .create(false)
                .open(in_path)
                .context("Cannot open file for writing!")?;
            reencode(std::io::Cursor::new(file_in_mem), file, args)?;
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
            reencode(in_file, out_file, args)?;
        }
        (None, _) => {
            bail!("Input filename must be specified!");
        }
    }

    Ok(())
}

fn reencode<R: Read + Seek, W: Write + Seek>(reader: R, writer: W, args: &ReencodeArgs) -> AnyResult<()> {
    let mut buf_r = Vec::new();
    let mut buf_w = Vec::new();
    let mut scratch = Vec::new();

    let mut mfr = MwFileReader::new(reader, &mut buf_r)
        .context("Failed to load input file as a MineWars format file!")?;

    if !args.ignore_checksums {
        mfr.verify_checksums()
            .context("Checksum verification failed!")?;
    }

    let (b_file, b_is) = MwFileBuilder::new(writer, &mut buf_w)?
        .start_is()?;

    let (mfr, mut isr) = mfr.read_is()?;
    let map: MapDataTopo<MapGenTileData> = isr.read_map_dyntopo(Some(&mut scratch), true)?;

    let compress_map = match (isr.is_mapdata_compressed(), args.compress_map, args.decompress_map) {
        (_, true, true) => bail!("--compress-map and --decompress-map cannot both be specified!"),
        (true, _, false) => true,
        (true, false, true) => false,
        (false, true, false) => true,
        (false, false, _) => false,
    };

    let b_is = if compress_map {
        match map {
            MapDataTopo::Hex(map) => {
                b_is.with_map_lz4compressed(&map, true, &mut scratch)?
            }
            MapDataTopo::Sq(map) => {
                b_is.with_map_lz4compressed(&map, true, &mut scratch)?
            }
        }
    } else {
        match map {
            MapDataTopo::Hex(map) => {
                b_is.with_map_uncompressed(&map, true)?
            }
            MapDataTopo::Sq(map) => {
                b_is.with_map_uncompressed(&map, true)?
            }
        }
    };

    let cits = isr.read_cits()?;
    let b_is = b_is.with_cits(cits.iter().cloned())?;

    let b_is = if args.anonymize || isr.is_anonymized() {
        b_is.with_anonymous_players(isr.n_players())?
    } else {
        b_is.with_named_players(isr.read_players()?)?
    };

    // TODO: rules
    let b_is = b_is.with_rules()?;

    let b_file = b_file.with_is(b_is.finish()?)?;

    // TODO: frames
    b_file.finish()?;

    Ok(())
}
