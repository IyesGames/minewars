use mw_common::game::{MapGenTileData, TileKind};
use mw_common::grid::*;
use mw_dataformat::read::MwFileReader;

use crate::prelude::*;
use crate::{CommonArgs, MapAsciiArgs};

pub fn main(common: &CommonArgs, _args: &MapAsciiArgs) -> AnyResult<()> {
    let file = if let Some(in_path) = &common.input {
        std::fs::OpenOptions::new()
            .read(true)
            .open(in_path)
            .context("Cannot open input file!")?
    } else {
        bail!("Input filename must be specified!");
    };
    let mut buf = Vec::new();
    let mut scratch = Vec::new();

    let mfr = MwFileReader::new(file, &mut buf)
        .context("Failed to load input file as a MineWars format file!")?;

    let (_, mut isr) = mfr.read_is()?;

    fn f_tile_ascii(cits: &[Pos], pos: Pos, kind: TileKind) -> u8 {
        if cits.iter().position(|p| *p == pos).is_some() {
            b'C'
        } else {
            match kind {
                TileKind::Water => b'~',
                TileKind::Regular => b'.',
                TileKind::Fertile => b',',
                TileKind::Forest => b'i',
                TileKind::Mountain => b'm',
                TileKind::Destroyed => b'+',
                TileKind::FoundationRoad => b'x',
                TileKind::FoundationStruct => b'_',
            }
        }
    }

    match isr.map_topology() {
        Topology::Hex => {
            let map: MapDataC<Hex, MapGenTileData> =
                isr.read_map(Some(&mut scratch), true)?;
            let cits = isr.read_cits_pos()?;
            map.ascii_art(&mut std::io::stdout().lock(), |c, d| f_tile_ascii(cits, c.into(), d.kind()))?;
        }
        Topology::Sq => {
            let map: MapDataC<Sq, MapGenTileData> =
                isr.read_map(Some(&mut scratch), true)?;
            let cits = isr.read_cits_pos()?;
            map.ascii_art(&mut std::io::stdout().lock(), |c, d| f_tile_ascii(cits, c.into(), d.kind()))?;
        }
    }

    Ok(())
}
