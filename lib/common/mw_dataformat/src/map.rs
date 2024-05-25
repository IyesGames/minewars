//! Encoding and Decoding of MineWars Map Data

use mw_common::game::MapGenTileData;
use thiserror::Error;

use mw_common::prelude::*;
use mw_common::{game::{ItemKind, TileKind}, grid::*};

/// Implement for types that can be serialized as a MW map.
pub trait MapTileDataOut: Sized {
    fn kind(&self) -> TileKind;
    fn item(&self) -> ItemKind;
    fn region(&self) -> u8;
}

/// Implement for types that can be deserialized as a MW map.
pub trait MapTileDataIn: Default + Sized {
    fn set_kind(&mut self, kind: TileKind);
    fn set_item(&mut self, kind: ItemKind);
    fn set_region(&mut self, region: u8);
}

impl MapTileDataOut for MapGenTileData {
    fn kind(&self) -> TileKind {
        MapGenTileData::kind(self)
    }
    fn item(&self) -> ItemKind {
        MapGenTileData::item(self)
    }
    fn region(&self) -> u8 {
        MapGenTileData::region(self)
    }
}

impl MapTileDataIn for MapGenTileData {
    fn set_kind(&mut self, kind: TileKind) {
        MapGenTileData::set_kind(self, kind);
    }

    fn set_item(&mut self, kind: ItemKind) {
        MapGenTileData::set_item(self, kind);
    }

    fn set_region(&mut self, region: u8) {
        MapGenTileData::set_region(self, region);
    }
}

/// Error when deserializing map data.
#[derive(Debug, Error)]
pub enum MapDecodeError {
    /// The number of bytes in the input buffer does not match
    /// what is expected based on the map's area (as per the map size)
    #[error("Map data length wrong for map size")]
    BadSize,
    /// The data in the input buffer is not valid
    #[error("Map data invalid")]
    BadData,
    /// LZ4 decompression failed
    #[error("Cannot decompress map data: {0}")]
    BadCompression(lz4_flex::block::DecompressError),
}

/// Encode map data in uncompressed form.
///
/// The binary map data will be appended to `out`.
pub fn serialize_map_uncompressed<C: Coord, D: MapTileDataOut, L: MapDataLayout<C>>(
    data: &MapData<C, D, L>,
    include_items: bool,
    out: &mut Vec<u8>,
) {
    // preallocate: 1 byte per tile + 1 byte per region = 2 * map_area
    out.reserve(2 * C::map_area(data.size()));
    // encode tiles
    {
        let d = &data[C::origin()];
        let mut byte = d.kind() as u8;
        if include_items {
            byte |= (d.item() as u8) << 4;
        }
        out.push(byte);
    }
    for ring in 1..=data.size() {
        for c in C::origin().iter_ring(ring) {
            let d = &data[c];
            let mut byte = d.kind() as u8;
            if include_items {
                byte |= (d.item() as u8) << 4;
            }
            out.push(byte);
        }
    }
    // encode regions
    {
        let d = &data[C::origin()];
        out.push(d.region());
    }
    for ring in 1..=data.size() {
        for c in C::origin().iter_ring(ring) {
            let d = &data[c];
            out.push(d.region());
        }
    }
}

/// Encode map data with LZ4 compression.
///
/// The compressed map data will be appended to `out`.
///
/// A `scratch` buffer must be provided (to help reuse allocations).
/// This function will clear it before and after use.
pub fn serialize_map_lz4compressed<C: Coord, D: MapTileDataOut, L: MapDataLayout<C>>(
    data: &MapData<C, D, L>,
    include_items: bool,
    out: &mut Vec<u8>,
    scratch: &mut Vec<u8>,
) {
    scratch.clear();
    serialize_map_uncompressed(data, include_items, scratch);

    let out_start = out.len();
    out.resize(out_start + lz4_flex::block::get_maximum_output_size(scratch.len()), 0);
    let compr_len = lz4_flex::block::compress_into(&scratch, &mut out[out_start..])
        .expect("LZ4 compression bug");
    out.truncate(out_start + compr_len);

    scratch.clear();
}

/// Decode map data from uncompressed data.
///
/// The size of the input must be appropriate for the map size.
/// The `data` will be mutated in-place by setting the new values.
pub fn deserialize_map_uncompressed<C: Coord, D: MapTileDataIn, L: MapDataLayout<C>>(
    data: &mut MapData<C, D, L>,
    include_items: bool,
    mut input: &[u8],
) -> Result<(), MapDecodeError> {
    if input.len() != 2 * C::map_area(data.size()) {
        return Err(MapDecodeError::BadSize);
    }
    // decode tiles
    {
        let d = &mut data[C::origin()];
        d.set_kind(
            TileKind::from_u8((input[0] & 0x0F) >> 0)
                .ok_or(MapDecodeError::BadData)?
        );
        if include_items {
            d.set_item(
                ItemKind::from_u8((input[0] & 0xF0) >> 4)
                    .ok_or(MapDecodeError::BadData)?
            );
        }
        input = &input[1..];
    }
    for ring in 1..=data.size() {
        for c in C::origin().iter_ring(ring) {
            let d = &mut data[c];
            d.set_kind(
                TileKind::from_u8((input[0] & 0x0F) >> 0)
                    .ok_or(MapDecodeError::BadData)?
            );
            if include_items {
                d.set_item(
                    ItemKind::from_u8((input[0] & 0xF0) >> 4)
                        .ok_or(MapDecodeError::BadData)?
                );
            }
            input = &input[1..];
        }
    }
    // decode regions
    {
        let d = &mut data[C::origin()];
        d.set_region(input[0]);
        input = &input[1..];
    }
    for ring in 1..=data.size() {
        for c in C::origin().iter_ring(ring) {
            let d = &mut data[c];
            d.set_region(input[0]);
            input = &input[1..];
        }
    }
    Ok(())
}

/// Decode map data from LZ4 compressed data.
///
/// The size of the input must be appropriate for the map size.
/// The `data` will be mutated in-place by setting the new values.
///
/// A `scratch` buffer must be provided (to help reuse allocations).
/// This function will clear it before and after use.
pub fn deserialize_map_lz4compressed<C: Coord, D: MapTileDataIn, L: MapDataLayout<C>>(
    data: &mut MapData<C, D, L>,
    include_items: bool,
    input: &[u8],
    scratch: &mut Vec<u8>,
) -> Result<(), MapDecodeError> {
    scratch.clear();
    scratch.resize(2 * C::map_area(data.size()), 0);
    let data_len = lz4_flex::block::decompress_into(input, scratch)
        .map_err(MapDecodeError::BadCompression)?;
    scratch.truncate(data_len);

    deserialize_map_uncompressed(data, include_items, scratch)?;
    scratch.clear();
    Ok(())
}
