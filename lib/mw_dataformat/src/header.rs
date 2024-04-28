//! Working with the Header of a MineWars file/stream.
//!
//! This is the first part of the Initialization Sequence
//! (the section contatining game metadata, map data, etc.,
//! before the Game Update Messages begin) which helps
//! navigate the contents of the file.

use mw_common::grid::*;

#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct Version(pub u8, pub u8, pub u8, pub u8);

/// The Initialization Sequence Header
///
/// Used in network streams. Part of the file header.
#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct ISHeader {
    pub version: Version,
    pub flags: u8,
    pub map_size: u8,
    pub n_players: u8,
    pub n_regions: u8,
    pub len_mapdata_compressed: u32,
    pub len_rules: u16,
    pub len_citdata_names: u16,
    pub len_playerdata: u16,
    pub reserved0: u16,
}

/// The MineWars File Header Extras
///
/// Contains  checksums and frame data length.
#[derive(Debug, Clone, Copy)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, packed)]
pub struct MwFileHeader {
    pub checksum_header: u64,
    pub checksum_is: u64,
    pub checksum_framedata: u64,
    pub len_framedata_compressed: u32,
    pub len_framedata_raw: u32,
}

impl Default for ISHeader {
    fn default() -> Self {
        let mut r: ISHeader = bytemuck::Zeroable::zeroed();
        r.version = crate::FORMAT_VERSION;
        r
    }
}

impl Default for MwFileHeader {
    fn default() -> Self {
        bytemuck::Zeroable::zeroed()
    }
}

impl ISHeader {
    const FLAG_TOPOLOGY_MASK: u8 = 0b00001000;
    const FLAG_TOPOLOGY_SHIFT: u8 = 3;

    pub fn version_is_compatible(&self, version: Version) -> bool {
        version.0 == self.version.0 &&
        version.1 == self.version.1 &&
        version.2 == self.version.2
        // 3 is allowed to change, to signify compatible versions
    }
    pub fn map_topology(&self) -> Topology {
        if self.flags & Self::FLAG_TOPOLOGY_MASK == 0 {
            Topology::Hex
        } else {
            Topology::Sq
        }
    }
    pub fn set_map_topology(&mut self, t: Topology) {
        let flag = match t {
            Topology::Hex => 0,
            Topology::Sq => 1,
        };
        self.flags = (self.flags & !Self::FLAG_TOPOLOGY_MASK) | (flag << Self::FLAG_TOPOLOGY_SHIFT);
    }
    pub fn serialized_len() -> usize {
        core::mem::size_of::<Self>()
    }
    pub fn serialize(&self, out: &mut Vec<u8>) {
        let start = out.len();
        out.reserve(Self::serialized_len());
        out.extend_from_slice(bytemuck::bytes_of(self));
        // ensure endianness
        #[cfg(target_endian = "little")]
        {
            let out_header: &mut ISHeader = bytemuck::from_bytes_mut(&mut out[start..]);
            out_header.len_rules = out_header.len_rules.swap_bytes();
            out_header.len_playerdata = out_header.len_playerdata.swap_bytes();
            out_header.len_mapdata_compressed = out_header.len_mapdata_compressed.swap_bytes();
        }
    }
    pub fn deserialize(input: &[u8]) -> Self {
        let mut out_header: ISHeader = *bytemuck::from_bytes(input);
        // ensure endianness
        #[cfg(target_endian = "little")]
        {
            out_header.len_rules = out_header.len_rules.swap_bytes();
            out_header.len_playerdata = out_header.len_playerdata.swap_bytes();
            out_header.len_mapdata_compressed = out_header.len_mapdata_compressed.swap_bytes();
        }
        out_header
    }
    pub fn is_mapdata_compressed(&self) -> bool {
        self.len_mapdata_compressed() != self.len_mapdata_raw()
    }
    pub fn is_anonymized(&self) -> bool {
        self.len_playerdata() == 0
    }
    pub fn len_total_is(&self) -> usize {
        Self::serialized_len() + self.len_total_data()
    }
    pub fn len_rules(&self) -> usize {
        self.len_rules as usize
    }
    pub fn len_mapdata_compressed(&self) -> usize {
        self.len_mapdata_compressed as usize
    }
    pub fn len_mapdata_raw(&self) -> usize {
        2 * match self.map_topology() {
            Topology::Hex => Hex::map_area(self.map_size),
            Topology::Sq => Sq::map_area(self.map_size),
        }
    }
    pub fn len_playerdata(&self) -> usize {
        self.len_playerdata as usize
    }
    pub fn len_citdata_names(&self) -> usize {
        self.len_citdata_names as usize
    }
    pub fn len_citdata_pos(&self) -> usize {
        2 * self.n_regions as usize
    }
    pub fn offset_mapdata(&self) -> usize {
        0
    }
    pub fn offset_citdata_pos(&self) -> usize {
        self.len_mapdata_compressed()
    }
    pub fn offset_citdata_names(&self) -> usize {
        self.len_mapdata_compressed()
        + self.len_citdata_pos()
    }
    pub fn offset_playerdata(&self) -> usize {
        self.len_mapdata_compressed()
        + self.len_citdata_pos()
        + self.len_citdata_names()
    }
    pub fn offset_rules(&self) -> usize {
        self.len_mapdata_compressed()
        + self.len_citdata_pos()
        + self.len_citdata_names()
        + self.len_playerdata()
    }
    pub fn len_total_data(&self) -> usize {
        self.len_mapdata_compressed()
        + self.len_citdata_pos()
        + self.len_citdata_names()
        + self.len_playerdata()
        + self.len_rules()
    }
}

impl MwFileHeader {
    pub fn serialized_len() -> usize {
        core::mem::size_of::<Self>()
    }
    pub fn serialize(&self, out: &mut Vec<u8>) {
        let start = out.len();
        out.reserve(Self::serialized_len());
        out.extend_from_slice(bytemuck::bytes_of(self));
        // ensure endianness
        #[cfg(target_endian = "little")]
        {
            let out_header: &mut MwFileHeader = bytemuck::from_bytes_mut(&mut out[start..]);
            out_header.checksum_header = out_header.checksum_header.swap_bytes();
            out_header.checksum_is = out_header.checksum_is.swap_bytes();
            out_header.checksum_framedata = out_header.checksum_framedata.swap_bytes();
            out_header.len_framedata_compressed = out_header.len_framedata_compressed.swap_bytes();
            out_header.len_framedata_raw = out_header.len_framedata_raw.swap_bytes();
        }
    }
    pub fn deserialize(input: &[u8]) -> Self {
        let mut out_header: MwFileHeader = *bytemuck::from_bytes(input);
        // ensure endianness
        #[cfg(target_endian = "little")]
        {
            out_header.checksum_header = out_header.checksum_header.swap_bytes();
            out_header.checksum_is = out_header.checksum_is.swap_bytes();
            out_header.checksum_framedata = out_header.checksum_framedata.swap_bytes();
            out_header.len_framedata_compressed = out_header.len_framedata_compressed.swap_bytes();
            out_header.len_framedata_raw = out_header.len_framedata_raw.swap_bytes();
        }
        out_header
    }
    pub fn is_framedata_compressed(&self) -> bool {
        self.len_framedata_compressed() != self.len_framedata_raw()
    }
    pub fn len_framedata_compressed(&self) -> usize {
        self.len_framedata_compressed as usize
    }
    pub fn len_framedata_raw(&self) -> usize {
        self.len_framedata_raw as usize
    }
    pub fn checksummable_start_offset() -> usize {
        std::mem::size_of::<u64>()
    }
}
