use bitflags::bitflags;
use zerocopy::FromBytes;

use crate::format::{InitSeqHeader, FileHeader};

pub enum DecodeError {
    /// DecodeFlags set to invalid value
    BadFlags,
    /// Protocol is a different version
    Version,
    /// Checksum error (likely data corruption)
    Checksum,
    /// Decompression error
    Compression,
    /// Error with the file format
    Format,
    /// File does not contain as much data as it says it does
    DataLength,
    /// Usage of reserved fields
    Reserved,
    /// Malformed update message
    OpCode,
}

bitflags! {
    #[repr(transparent)]
    /// Settings for the decoder
    pub struct DecodeFlags: u32 {
        /// Ignore invalid checksums (dangerous)
        const NO_CHECKSUMS = 0b00000001;
        /// Ignore usage of reserved fields
        const ALLOW_RESERVED = 0b00000010;
        /// Only decode the map data, no update frames
        const MAP_ONLY = 0b00000100;
        /// Only decode metadata
        const METADATA = 0b00001000;
        /// Only header metadata; no frames
        const HEADER_ONLY = 0b00001100;
    }
}

impl Default for DecodeFlags {
    fn default() -> Self {
        DecodeFlags::from_bits_truncate(0u32)
    }
}

/// Use this work with a single "view": a stream of player protocol messages
pub struct ViewDecoder<'data> {
    flags: DecodeFlags,
    data: &'data [u8],
}

/// Use this to work with a frame: multiplexed views
pub struct FrameDecoder<'data> {
    flags: DecodeFlags,
    data: &'data [u8],
}

/// Full decoder for a spectator stream
///
/// Owns the data.
pub struct SpectDecoder {
    flags: DecodeFlags,
    data: Vec<u8>,
}

impl SpectDecoder {
}

/// Use this if you have loaded a replay file
///
/// This will handle things specific to the file format, like
/// decompression and checksums. It needs to be converted into a
/// `SpectDecoder` to actually read the data.
pub struct ReplayDecoder {
    flags: DecodeFlags,
    data: Vec<u8>,
}

impl ReplayDecoder {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, DecodeError> {
        Self::from_bytes_with_flags(DecodeFlags::default(), data)
    }

    pub fn from_bytes_with_flags(flags: DecodeFlags, data: Vec<u8>) -> Result<Self, DecodeError> {
        let r = Self {
            flags,
            data,
        };

        r.validate()?;

        Ok(r)
    }

    pub fn into_spect(self) -> Result<SpectDecoder, DecodeError> {
        unimplemented!()
    }

    /// Perform checksum verification
    ///
    /// This can be manually called even if the decoder
    /// was created with `DecodeFlags::NO_CHECKSUMS`.
    pub fn verify_checksums(&self) -> Result<(), DecodeError> {
        let file_header = self.file_header();

        // the header checksum covers the file + init sequence headers
        // but we need to skip the first checksum (8 bytes)
        let initseq_data_start = FileHeader::LEN + InitSeqHeader::LEN;
        let header_data = &self.data[8 .. initseq_data_start];
        let header_checksum = seahash::hash(header_data);
        if header_checksum != file_header.checksum_header.get() {
            return Err(DecodeError::Checksum);
        }

        if self.flags.contains(DecodeFlags::HEADER_ONLY) {
            return Ok(());
        }

        // figure out the data payload length
        let initseq_header = self.initseq_header();
        let initseq_data_len = initseq_header.initseq_data_len();
        let frames_start = initseq_data_start + initseq_data_len;
        let initseq_data = &self.data[initseq_data_start .. frames_start];
        let initseq_data_checksum = seahash::hash(initseq_data);
        if initseq_data_checksum != file_header.checksum_initseq_data.get() {
            return Err(DecodeError::Checksum);
        }

        if self.flags.contains(DecodeFlags::MAP_ONLY) {
            return Ok(());
        }

        let frames_len = file_header.frames_len_compressed.get() as usize;
        let frames_data = &self.data[frames_start .. (frames_start + frames_len)];
        let frames_checksum = seahash::hash(frames_data);
        if frames_checksum != file_header.checksum_frames.get() {
            return Err(DecodeError::Checksum);
        }

        Ok(())
    }

    /// Check that everything is sane
    fn validate(&self) -> Result<(), DecodeError> {
        let mut expected_len = FileHeader::LEN + InitSeqHeader::LEN;
        if self.data.len() < expected_len {
            return Err(DecodeError::DataLength);
        }

        let file_header = self.file_header();
        file_header.validate()?;

        let initseq_header = self.initseq_header();
        initseq_header.validate()?;

        if !self.flags.contains(DecodeFlags::HEADER_ONLY) {
            expected_len += initseq_header.mapdata_len_compressed.get() as usize;
        }

        if !self.flags.contains(DecodeFlags::MAP_ONLY) {
            expected_len += file_header.frames_len_compressed.get() as usize;
        }

        if self.data.len() < expected_len {
            return Err(DecodeError::DataLength);
        }

        if !self.flags.contains(DecodeFlags::NO_CHECKSUMS) {
            self.verify_checksums()?;
        }

        Ok(())
    }

    fn file_header(&self) -> FileHeader {
        FileHeader::read_from(
            &self.data[..FileHeader::LEN]
        ).unwrap()
    }

    fn initseq_header(&self) -> InitSeqHeader {
        let initseq_data_start = FileHeader::LEN + InitSeqHeader::LEN;
        InitSeqHeader::read_from(
            &self.data[FileHeader::LEN .. initseq_data_start]
        ).unwrap()
    }
}

