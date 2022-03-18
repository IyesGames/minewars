use bitflags::bitflags;
use zerocopy::{FromBytes, AsBytes};

pub mod int {
    #![allow(non_camel_case_types)]
    #![allow(dead_code)]

    use zerocopy::byteorder::{U16, U32, U64, BigEndian};

    pub type u16be = U16<BigEndian>;
    pub type u32be = U32<BigEndian>;
    pub type u64be = U64<BigEndian>;
}

use int::*;

use crate::dec::DecodeError;

bitflags! {
    #[derive(FromBytes, AsBytes)]
    #[repr(transparent)]
    pub struct InitSeqFlags: u8 {
        const RESERVED = 0b11111111;
    }
}

#[derive(FromBytes, AsBytes)]
#[repr(C)]
pub struct GameParams {
    pub res_base: u8,
    pub res_land: u8,
    pub res_fertile: u8,
    pub res_mtn: u8,
    pub radii: u16be,
    pub cost_road: u16be,
    pub cost_mine: u16be,
    pub cost_decoy: u16be,
}

#[derive(FromBytes, AsBytes)]
#[repr(C)]
pub struct InitSeqHeader {
    pub flags: InitSeqFlags,
    pub version: u8,
    pub tickrate: u8,
    pub map_size: u8,
    pub n_players: u8,
    pub n_cities: u8,
    pub playernames_len: u16be,
    pub mapdata_len_compressed: u16be,
    pub mapdata_len_uncompressed: u16be,
    pub game_params: GameParams,
}

#[derive(FromBytes, AsBytes)]
#[repr(C)]
pub struct FileHeader {
    pub checksum_header: u64be,
    pub checksum_initseq_data: u64be,
    pub checksum_frames: u64be,
    pub frames_len_compressed: u16be,
    pub frames_len_uncompressed: u16be,
}

#[derive(FromBytes, AsBytes)]
#[repr(C)]
pub struct FrameHeader {
    pub delta: u16be,
    pub data_len: u8,
    pub plids: u8,
}

#[repr(u8)]
pub enum FrameKind {
    Hetero = 0,
    Homo = 1,
}

impl InitSeqHeader {
    pub const LEN: usize = std::mem::size_of::<Self>();

    /// Compute the length of the data in the initialization sequence
    pub fn initseq_data_len(&self) -> usize {
        0usize
            + self.playernames_len.get() as usize
            + self.n_cities as usize * 2
            + self.mapdata_len_compressed.get() as usize
    }

    pub fn validate(&self) -> Result<(), DecodeError> {
        if self.version != crate::FORMAT_VERSION {
            return Err(DecodeError::Version);
        }
        if self.tickrate == 0
            || self.n_cities == 0
            || self.n_cities > 16
            || self.n_players == 0
            || self.n_players > 6
            || self.mapdata_len_compressed.get() > self.mapdata_len_uncompressed.get()
        {
            return Err(DecodeError::Format);
        }
        Ok(())
    }
}

impl FileHeader {
    pub const LEN: usize = std::mem::size_of::<Self>();

    pub fn validate(&self) -> Result<(), DecodeError> {
        if self.frames_len_compressed.get() > self.frames_len_uncompressed.get()
        {
            return Err(DecodeError::Format);
        }
        Ok(())
    }
}

impl FrameHeader {
    pub fn kind(&self) -> FrameKind {
        if self.plids & 0b10000000 != 0 {
            FrameKind::Homo
        } else {
            FrameKind::Hetero
        }
    }
}
