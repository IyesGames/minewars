//! Reading/Decoding MineWars Data Streams or Files

use mw_common::{grid::*, phoneme::Ph, plid::PlayerId};
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
    off_data: u64,
    current_time_ms: u64,
    max_plid: u8,
    n_views: u8,
    frame_kind: FrameKind,
    buf: &'b mut Vec<u8>,
    reader: R,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameKind {
    Unknown,
    Keepalive,
    Heterogenous,
    Homogenous,
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
                    off_data: 0,
                    current_time_ms: 0,
                    max_plid: self.is_header.max_plid(),
                    n_views: 0,
                    frame_kind: FrameKind::Unknown,
                    buf: self.buf,
                    reader: cursor,
                }
            ))
        } else {
            Ok(MwFrameReader::Uncompressed(
                MwFrameDataReader {
                    off_data: offset_framedata,
                    current_time_ms: 0,
                    max_plid: self.is_header.max_plid(),
                    n_views: 0,
                    frame_kind: FrameKind::Unknown,
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
    pub fn max_plid(&self) -> u8 {
        self.is_header.max_plid()
    }
    pub fn max_sub_plid(&self) -> u8 {
        self.is_header.max_sub_plid()
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
    pub fn read_map<'s, C, D, L>(
        &mut self,
        scratch: Option<&'s mut Vec<u8>>,
        include_items: bool,
    ) -> Result<MapData<C, D, L>, MwReaderError>
    where
        C: Coord,
        D: MapTileDataIn,
        L: MapDataLayout<C>,
    {
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
                scratch.resize(2 * C::map_area(self.map_size()), 0);
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
    pub fn read_cits_pos(&mut self) -> Result<&[Pos], MwReaderError> {
        self.buf.resize(self.is_header.len_citdata_pos(), 0);
        self.reader.seek(SeekFrom::Start(self.off_data as u64 + self.is_header.offset_citdata_pos() as u64))?;
        self.reader.read_exact(self.buf)?;
        Ok(bytemuck::cast_slice(&self.buf))
    }
    pub fn read_cits_names(&mut self) -> Result<CitNamesIter<'_>, MwReaderError> {
        self.buf.resize(self.is_header.len_citdata_names(), 0);
        self.reader.seek(SeekFrom::Start(self.off_data as u64 + self.is_header.offset_citdata_names() as u64))?;
        self.reader.read_exact(self.buf)?;
        Ok(CitNamesIter {
            current_cit: 0,
            total_cits: self.n_regions(),
            buf: self.buf
        })
    }
}

impl<'b, R: Read + Seek> MwFrameDataReader<'b, R> {
    pub fn current_time_ms(&self) -> u64 {
        self.current_time_ms
    }
    pub fn advance_next_frame(&mut self) -> Result<(), MwReaderError> {
        self.off_data += self.offset_next_frame();
        self.buf.resize(2, 0);
        self.reader.seek(SeekFrom::Start(self.off_data))?;
        self.reader.read_exact(self.buf)?;
        let h_delta = u16::from_be_bytes([self.buf[0], self.buf[1]]);
        if h_delta & !(1 << 15) == !(1 << 15) {
            // Keepalive frame
            self.current_time_ms += 1 << 15;
            self.frame_kind = FrameKind::Keepalive;
        } else if h_delta & (1 << 15) != 0 {
            // Homogenous frame
            self.current_time_ms += (h_delta & !(1 << 15)) as u64;
            self.frame_kind = FrameKind::Homogenous;
            let len_plidsmask = self.len_plidsmask();
            // Read the plidsmask + data length
            self.buf.resize(len_plidsmask + 1, 0);
            self.reader.read_exact(self.buf)?;
            self.n_views = 0; // not used for homo frames
            // Read the data, so we can cache it
            let len_data = self.buf[len_plidsmask] as usize + 1;
            self.buf.resize(len_plidsmask + 1 + len_data, 0);
            self.reader.read_exact(&mut self.buf[(len_plidsmask + 1)..])?;
        } else {
            // Heterogenous frame
            self.current_time_ms += (h_delta & !(1 << 15)) as u64;
            self.frame_kind = FrameKind::Heterogenous;
            let len_plidsmask = self.len_plidsmask();
            self.buf.resize(len_plidsmask, 0);
            self.reader.read_exact(self.buf)?;
            self.n_views = match len_plidsmask {
                1 => self.buf[0].count_ones() as u8,
                2 => u16::from_ne_bytes([self.buf[0], self.buf[1]]).count_ones() as u8,
                _ => unreachable!(),
            };
            self.buf.resize(len_plidsmask + self.n_views as usize, 0);
            self.reader.read_exact(&mut self.buf[len_plidsmask..])?;
        }
        Ok(())
    }
    pub fn iter_streams(&mut self) -> MwFrameStreamIter<'_, 'b, R> {
        MwFrameStreamIter {
            i: 0,
            b_mask: 0,
            i_mask: self.len_plidsmask(),
            i_stream: 0,
            off_stream: self.len_plidsmask() as u64 + self.n_views as u64,
            reader: self,
        }
    }
    pub fn get_player_stream(&mut self, plid: PlayerId) -> Result<&'_ [u8], MwReaderError> {
        match self.frame_kind {
            FrameKind::Unknown | FrameKind::Keepalive => Ok(&[]),
            FrameKind::Homogenous => {
                if !self.contains_view(plid) {
                    return Ok(&[]);
                }
                let offset = self.len_plidsmask() + 1;
                let data = &self.buf[offset..];
                Ok(data)
            }
            FrameKind::Heterogenous => {
                if !self.contains_view(plid) {
                    return Ok(&[]);
                }
                let plid = u8::from(plid);
                let offset_lens = self.len_plidsmask();
                let base_len = self.len_plidsmask() + self.n_views as usize;
                let mut offset_stream = base_len as u64;
                let len_stream;
                let mut buf_lens = &self.buf[offset_lens..];
                let mut i = 0;
                loop {
                    if i == plid {
                        len_stream = buf_lens[0] as usize + 1;
                        break;
                    }
                    let mask_byte = self.len_plidsmask() - i as usize / 8; // Big Endian
                    let mask_bit = i % 8;
                    if self.buf[mask_byte] & (1 << mask_bit) != 0 {
                        offset_stream += buf_lens[0] as u64 + 1;
                        buf_lens = &buf_lens[1..];
                    }
                    i += 1;
                }
                self.buf.resize(base_len + len_stream, 0);
                self.reader.seek(SeekFrom::Start(self.off_data + offset_stream))?;
                self.reader.read_exact(&mut self.buf[base_len..])?;
                Ok(&self.buf[base_len..])
            }
        }
    }
    fn len_plidsmask(&self) -> usize {
        (self.max_plid as usize + 1) / 8 + 1
    }
    fn offset_next_frame(&self) -> u64 {
        let len_plidsmask = self.len_plidsmask();
        match self.frame_kind {
            FrameKind::Unknown => 0,
            FrameKind::Keepalive => 2,
            FrameKind::Heterogenous => {
                let mut len_data = 0;
                for b in &self.buf[len_plidsmask..(len_plidsmask + self.n_views as usize)] {
                    len_data += *b as u64 + 1;
                }
                2 + len_plidsmask as u64 + self.n_views as u64 + len_data
            },
            FrameKind::Homogenous => {
                let len_data = self.buf[len_plidsmask] as u64 + 1;
                3 + len_plidsmask as u64 + len_data
            },
        }
    }
    pub fn contains_view(&self, plid: PlayerId) -> bool {
        match self.frame_kind {
            FrameKind::Unknown | FrameKind::Keepalive => return false,
            _ => {}
        }
        let plid = u8::from(plid);
        if plid > self.max_plid {
            return false;
        }
        let mask_byte = self.len_plidsmask() - plid as usize / 8; // Big Endian
        let mask_bit = plid % 8;
        self.buf[mask_byte] & (1 << mask_bit) != 0
    }
    pub fn frame_kind(&self) -> FrameKind {
        self.frame_kind
    }
}

pub struct MwFrameStreamIter<'a, 'b, R: Read + Seek> {
    i: usize,
    b_mask: u8,
    i_mask: usize,
    i_stream: usize,
    off_stream: u64,
    reader: &'a mut MwFrameDataReader<'b, R>,
}

impl<'a, 'b, R: Read + Seek> Iterator for MwFrameStreamIter<'a, 'b, R> {
    type Item = Result<&'a [u8], MwReaderError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i > self.reader.max_plid as usize {
            return None;
        }
        let i_sub = self.i % 8;
        self.i += 1;
        match self.reader.frame_kind {
            FrameKind::Unknown => Some(Ok(&[])),
            FrameKind::Keepalive => Some(Ok(&[])),
            FrameKind::Heterogenous => {
                let len_plidsmask = self.reader.len_plidsmask();
                if i_sub == 0 {
                    if self.i_mask == 0 {
                        return None;
                    }
                    self.i_mask -= 1;
                    self.b_mask = self.reader.buf[self.i_mask];
                }
                if self.b_mask & (1 << i_sub) != 0 {
                    let base_len = self.reader.len_plidsmask() + self.reader.n_views as usize;
                    let len_stream = self.reader.buf[len_plidsmask + self.i_stream] as usize + 1;
                    self.i_stream += 1;
                    self.reader.buf.resize(base_len + len_stream, 0);
                    let r = self.reader.reader.seek(SeekFrom::Start(self.reader.off_data + self.off_stream))
                        .and_then(|_| self.reader.reader.read_exact(&mut self.reader.buf[base_len..]));
                    self.off_stream += len_stream as u64;
                    match r {
                        Ok(_) => unsafe {
                            // transmute lifetimes
                            // SAFETY: `&mut self` is based on the `reader`
                            // which carries the correct lifetime. The data in `buf`
                            // cannot outlive `self`
                            Some(Ok(
                                std::mem::transmute::<&'_ [u8], &'a [u8]>(&self.reader.buf[base_len..])
                            ))
                        }
                        Err(e) => {
                            Some(Err(e.into()))
                        }
                    }
                } else {
                    Some(Ok(&[]))
                }
            },
            FrameKind::Homogenous => {
                let len_plidsmask = self.reader.len_plidsmask();
                if i_sub == 0 {
                    if self.i_mask == 0 {
                        return None;
                    }
                    self.i_mask -= 1;
                    self.b_mask = self.reader.buf[self.i_mask];
                }
                if self.b_mask & (1 << i_sub) != 0 {
                    let offset = len_plidsmask + 1;
                    unsafe {
                        // transmute lifetimes
                        // SAFETY: `&mut self` is based on the `reader`
                        // which carries the correct lifetime. The data in `buf`
                        // cannot outlive `self`
                        Some(Ok(
                            std::mem::transmute::<&'_ [u8], &'a [u8]>(&self.reader.buf[offset..])
                        ))
                    }
                } else {
                    Some(Ok(&[]))
                }
            },
        }
    }
}

impl<'a, 'b, R: Read + Seek> FusedIterator for MwFrameStreamIter<'a, 'b, R> {}

pub struct CitNamesIter<'b> {
    current_cit: u8,
    total_cits: u8,
    buf: &'b [u8],
}

impl<'b> Iterator for CitNamesIter<'b> {
    type Item = &'b [Ph];
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_cit >= self.total_cits {
            return None;
        }
        self.current_cit += 1;
        if self.buf.is_empty() {
            return Some(&[]);
        }
        let strlen = (self.buf[0] as usize).min(self.buf.len() - 1);
        self.buf = &self.buf[1..];
        let (out, rem) = self.buf.split_at(strlen);
        self.buf = rem;
        unsafe {
            Some(std::mem::transmute(out))
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}

impl<'b> ExactSizeIterator for CitNamesIter<'b> {
    fn len(&self) -> usize {
        (self.total_cits - self.current_cit) as usize
    }
}

impl<'b> FusedIterator for CitNamesIter<'b> {}
