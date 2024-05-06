use std::io::BufWriter;

use mw_common::{game::{MapGenTileData, TileKind}, grid::*, phoneme::Ph};

use crate::prelude::*;
use crate::{CommonArgs, GenMapArgs};

pub fn main(common: &CommonArgs, args: &GenMapArgs) -> AnyResult<()> {
    let file = if let Some(out_path) = &common.output {
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(out_path)
            .context("Cannot create output file!")?
    } else {
        bail!("Output filename must be specified!");
    };
    let mut buf = Vec::new();
    let mut scratch = Vec::new();
    let mut tile = MapGenTileData::default();
    tile.set_kind(TileKind::Regular);
    tile.set_region(0xFF);
    let map: MapDataC<Hex, _> = MapData::new(args.size, tile);

    let bufwriter = BufWriter::new(file);
    let (b_file, b_is) = mw_dataformat::write::MwFileBuilder::new(bufwriter, &mut buf)?
        .start_is()?;
    let is = b_is
        .with_map_lz4compressed(&map, true, &mut scratch)?
        .with_cits([
            (Pos(12, 17), [Ph::A, Ph::B, Ph::E, Ph::Z].as_slice()),
            (Pos(7, 3), [Ph::I, Ph::D, Ph::A].as_slice()),
        ])?
        .with_named_players(["iyes", "georgie", "gr.NET"])?
        .finish()?;
    let b_file = b_file.with_is(is)?;
    b_file.finish()?;

    Ok(())
}
