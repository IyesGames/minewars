//! Reading/Decoding MineWars Data Streams or Files

use mw_common::grid::*;
use thiserror::Error;

use std::{io::{Cursor, Read, Seek, SeekFrom}, iter::FusedIterator};

use crate::{header::{ISHeader, MwFileHeader}, map::MapTileDataIn, FORMAT_VERSION};

#[derive(Debug, Error)]
pub enum ChecksumError {
    #[error("Corrupted headers.")]
    BadHeader,
    #[error("Corrupted IS data (map/citinfo/playerinfo/rules).")]
    BadIS,
    #[error("Corrupted frame data (gameplay messages).")]
    BadFrames,
}

#[derive(Debug, Error)]
pub enum MwReaderError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Format version is incompatible.")]
    VersionIncompatible,
    #[error("Checksum invalid: {0}")]
    Checksum(#[from] ChecksumError),
    #[error("Attempted to decode compressed data as uncompressed.")]
    DataIsCompressed,
    #[error("Error when decompressing data: {0}")]
    Compression(#[from] lz4_flex::block::DecompressError),
    #[error("Map data cannot be decoded: {0}")]
    Map(#[from] crate::map::MapDecodeError),
    #[error("Wrong grid topology (hex/sq).")]
    WrongTopology,
}

/// MineWars Decoder (for Full data / with file header)
///
/// The `R` parameter should typically be either a file or an in-memory buffer.
///
/// It is expected to contain at least a complete Initialization Sequence
pub struct MwFileReader<'b, R: Read + Seek> {
    buf: &'b mut Vec<u8>,
    reader: R,
    file_header: MwFileHeader,
    is_header: ISHeader,
}

pub struct MwFileReaderAwaitsIS {
    /// to prevent users from constructing the struct directly
    _a: ()
}

/// MineWars Decoder (for bare Initialization Sequence)
///
/// The `R` parameter should typically be either a file or an in-memory buffer.
///
/// It is expected to contain at least a complete Initialization Sequence
pub struct MwISReader<'b, R: Read + Seek> {
    buf: &'b mut Vec<u8>,
    reader: R,
    off_data: u32,
    is_header: ISHeader,
}

pub enum MwFrameReader<'b, 's, R: Read + Seek> {
    Uncompressed(MwFrameDataReader<'b, R>),
    Compressed(MwFrameDataReader<'b, Cursor<&'s mut Vec<u8>>>),
}

pub struct MwFrameDataReader<'b, R: Read + Seek> {
    buf: &'b mut Vec<u8>,
    reader: R,
}

impl<'b, R: Read + Seek> MwFileReader<'b, R> {
    pub fn new(mut reader: R, buf: &'b mut Vec<u8>) -> Result<Self, MwReaderError> {
        reader.seek(SeekFrom::Start(0))?;
        // read file header
        buf.resize(MwFileHeader::serialized_len(), 0);
        reader.read_exact(buf)?;
        let file_header = MwFileHeader::deserialize(buf);
        // read IS header
        buf.resize(ISHeader::serialized_len(), 0);
        reader.read_exact(buf)?;
        let is_header = ISHeader::deserialize(buf);

        if !is_header.version_is_compatible(FORMAT_VERSION) {
            return Err(MwReaderError::VersionIncompatible);
        }

        Ok(Self {
            buf,
            reader,
            file_header,
            is_header,
        })
    }
    pub fn into_inner(self) -> R {
        self.reader
    }
    pub fn file_header(&self) -> &MwFileHeader {
        &self.file_header
    }
    pub fn is_header(&self) -> &ISHeader {
        &self.is_header
    }
    pub fn read_is(mut self) -> Result<(MwFileReaderAwaitsIS, MwISReader<'b, R>), MwReaderError> {
        let off_data = MwFileHeader::serialized_len() as u32 + ISHeader::serialized_len() as u32;
        self.reader.seek(SeekFrom::Start(off_data as u64))?;
        Ok((
            MwFileReaderAwaitsIS {
                _a: (),
            },
            MwISReader {
                buf: self.buf,
                reader: self.reader,
                off_data,
                is_header: self.is_header,
            }
        ))
    }
    pub fn read_frames<'s>(mut self, scratch: Option<&'s mut Vec<u8>>) -> Result<MwFrameReader<'b, 's, R>, MwReaderError> {
        let offset_framedata = MwFileHeader::serialized_len() as u64 +
            self.is_header.len_total_is() as u64;
        self.reader.seek(SeekFrom::Start(offset_framedata))?;
        if self.is_framedata_compressed() {
            let cursor = if let Some(scratch) = scratch {
                self.buf.resize(self.file_header.len_framedata_raw(), 0);
                self.reader.read_exact(self.buf)?;
                scratch.clear();
                scratch.resize(self.file_header.len_framedata_compressed(), 0);
                let data_len = lz4_flex::block::decompress_into(self.buf, scratch)?;
                scratch.truncate(data_len);
                Cursor::new(scratch)
            } else {
                return Err(MwReaderError::DataIsCompressed);
            };
            Ok(MwFrameReader::Compressed(
                MwFrameDataReader {
                    buf: self.buf,
                    reader: cursor,
                }
            ))
        } else {
            Ok(MwFrameReader::Uncompressed(
                MwFrameDataReader {
                    buf: self.buf,
                    reader: self.reader,
                }
            ))
        }
    }
    pub fn is_framedata_compressed(&self) -> bool {
        self.file_header.is_framedata_compressed()
    }
    pub fn verify_checksum_header(&mut self) -> Result<(), MwReaderError> {
        if self.compute_new_checksum_header()? != self.file_header.checksum_header {
            return Err(ChecksumError::BadHeader.into());
        }
        Ok(())
    }
    pub fn verify_checksum_isdata(&mut self) -> Result<(), MwReaderError> {
        if self.compute_new_checksum_isdata()? != self.file_header.checksum_is {
            return Err(ChecksumError::BadIS.into());
        }
        Ok(())
    }
    pub fn verify_checksum_framedata(&mut self) -> Result<(), MwReaderError> {
        if self.compute_new_checksum_framedata()? != self.file_header.checksum_framedata {
            return Err(ChecksumError::BadFrames.into());
        }
        Ok(())
    }
    pub fn verify_checksums(&mut self) -> Result<(), MwReaderError> {
        self.verify_checksum_header()?;
        self.verify_checksum_isdata()?;
        self.verify_checksum_framedata()?;
        Ok(())
    }
    pub fn compute_new_checksum_header(&mut self) -> Result<u64, MwReaderError> {
        self.buf.clear();
        self.file_header.serialize(self.buf);
        self.is_header.serialize(self.buf);
        let skip = MwFileHeader::checksummable_start_offset();
        Ok(seahash::hash(&self.buf[skip..]))
    }
    pub fn compute_new_checksum_isdata(&mut self) -> Result<u64, MwReaderError> {
        self.reader.seek(SeekFrom::Start(
            MwFileHeader::serialized_len() as u64 +
            ISHeader::serialized_len() as u64
        ))?;
        self.buf.resize(self.is_header.len_total_data(), 0);
        self.reader.read_exact(self.buf)?;
        Ok(seahash::hash(self.buf))
    }
    pub fn compute_new_checksum_framedata(&mut self) -> Result<u64, MwReaderError> {
        self.reader.seek(SeekFrom::Start(
            MwFileHeader::serialized_len() as u64 +
            self.is_header.len_total_is() as u64
        ))?;
        // PERF: this should probably be streaming instead of loading the whole file into memory
        self.buf.resize(self.file_header.len_framedata_compressed(), 0);
        self.reader.read_exact(self.buf)?;
        Ok(seahash::hash(self.buf))
    }
    pub fn compute_and_force_update_checksums(&mut self) -> Result<(), MwReaderError> {
        self.file_header.checksum_framedata = self.compute_new_checksum_framedata()?;
        self.file_header.checksum_is = self.compute_new_checksum_isdata()?;
        self.file_header.checksum_header = self.compute_new_checksum_header()?;
        Ok(())
    }
}

impl<'b, R: Read + Seek> MwISReader<'b, R> {
    pub fn new(mut reader: R, buf: &'b mut Vec<u8>) -> Result<Self, MwReaderError> {
        // read the IS header
        buf.resize(ISHeader::serialized_len(), 0);
        reader.read_exact(buf)?;
        let is_header = ISHeader::deserialize(buf);
        // remember the data start position
        let off_data = reader.stream_position()? as u32;
        Ok(Self {
            buf,
            reader,
            off_data,
            is_header,
        })
    }
    pub fn into_inner(self) -> R {
        self.reader
    }
    pub fn header(&self) -> &ISHeader {
        &self.is_header
    }
    pub fn n_players(&self) -> u8 {
        self.is_header.n_players
    }
    pub fn n_regions(&self) -> u8 {
        self.is_header.n_regions
    }
    pub fn map_size(&self) -> u8 {
        self.is_header.map_size
    }
    pub fn map_topology(&self) -> Topology {
        self.is_header.map_topology()
    }
    pub fn is_mapdata_compressed(&self) -> bool {
        self.is_header.is_mapdata_compressed()
    }
    pub fn is_anonymized(&self) -> bool {
        self.is_header.is_anonymized()
    }
    pub fn read_map<'s, C: Coord, D: MapTileDataIn>(
        &mut self,
        scratch: Option<&'s mut Vec<u8>>,
        include_items: bool,
    ) -> Result<MapData<C, D>, MwReaderError> {
        if C::TOPOLOGY != self.map_topology() {
            return Err(MwReaderError::WrongTopology);
        }

        let mut mapdata = MapData::new_with(self.map_size(), |_| D::default());

        self.buf.resize(self.is_header.len_mapdata_compressed(), 0);
        self.reader.seek(SeekFrom::Start(self.off_data as u64 + self.is_header.offset_mapdata() as u64))?;
        self.reader.read_exact(self.buf)?;

        if self.is_mapdata_compressed() {
            if let Some(scratch) = scratch {
                scratch.clear();
                scratch.resize(2 * mapdata.map_area(), 0);
                let data_len = lz4_flex::block::decompress_into(self.buf, scratch)?;
                scratch.truncate(data_len);
                crate::map::deserialize_map_uncompressed(&mut mapdata, include_items, scratch)?;
            } else {
                return Err(MwReaderError::DataIsCompressed);
            }
        } else {
            crate::map::deserialize_map_uncompressed(&mut mapdata, include_items, self.buf)?;
        }

        Ok(mapdata)
    }
    pub fn read_map_dyntopo<'s, D: MapTileDataIn>(
        &mut self,
        scratch: Option<&'s mut Vec<u8>>,
        include_items: bool,
    ) -> Result<MapDataTopo<D>, MwReaderError> {
        match self.map_topology() {
            Topology::Hex => {
                let mapdata = self.read_map::<Hex, D>(scratch, include_items)?;
                Ok(mapdata.into())
            }
            Topology::Sq => {
                let mapdata = self.read_map::<Sq, D>(scratch, include_items)?;
                Ok(mapdata.into())
            }
        }
    }
    pub fn read_cits(&mut self) -> Result<&[Pos], MwReaderError> {
        self.buf.resize(self.is_header.len_citdata(), 0);
        self.reader.seek(SeekFrom::Start(self.off_data as u64 + self.is_header.offset_citdata() as u64))?;
        self.reader.read_exact(self.buf)?;
        Ok(bytemuck::cast_slice(&self.buf))
    }
    pub fn read_players(&mut self) -> Result<PlayerNamesIter<'_>, MwReaderError> {
        self.buf.resize(self.is_header.len_playerdata(), 0);
        self.reader.seek(SeekFrom::Start(self.off_data as u64 + self.is_header.offset_playerdata() as u64))?;
        self.reader.read_exact(self.buf)?;
        Ok(PlayerNamesIter {
            current_plid: 0,
            total_plids: self.n_players(),
            buf: self.buf
        })
    }
}

pub struct PlayerNamesIter<'b> {
    current_plid: u8,
    total_plids: u8,
    buf: &'b [u8],
}

impl<'b> Iterator for PlayerNamesIter<'b> {
    type Item = &'b str;
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_plid >= self.total_plids {
            return None;
        }
        self.current_plid += 1;
        if self.buf.is_empty() {
            return Some("");
        }
        let strlen = (self.buf[0] as usize).min(self.buf.len() - 1);
        let s = std::str::from_utf8(&self.buf[1..(strlen + 1)])
            .unwrap_or("");
        self.buf = &self.buf[(strlen + 1)..];
        Some(s)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'b> ExactSizeIterator for PlayerNamesIter<'b> {
    fn len(&self) -> usize {
        (self.total_plids - self.current_plid) as usize
    }
}

impl<'b> FusedIterator for PlayerNamesIter<'b> {}
