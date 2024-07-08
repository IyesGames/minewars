//! Writing/Encoding MineWars Data Streams or Files

use seahash::SeaHasher;
use thiserror::Error;
use std::{hash::Hasher, io::{Cursor, Seek, SeekFrom, Write}};
use mw_common::{grid::*, phoneme::Ph};

use crate::{header::{ISHeader, MwFileHeader}, map::MapTileDataOut};

#[derive(Debug, Error)]
pub enum MwWriterError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Builder for a full MineWars file
///
/// The general process of encoding a file is as follows (pseudocode):
///
/// ```rust
/// let (b_file, b_is) = MwFileBuilder::new(out)?.start_is()?;
/// let is = b_is
///   .with_map(...)?
///   .with_cits(...)?
///   .with_rules(...)?
///   .finish()?;
/// let mut b_file
///   .with_is(is)?;
/// for game_update in game_updates {
///   let (b_file2, b_frame) = b_file.start_frame(...)?;
///   let frame = b_frame
///     .with_msgs(game_update.msgs)?
///     .finish()?;
///   b_file = b_file2.with_frames(frame)?;
/// }
/// b_file.finish()?;
/// ```
///
/// Any parts of the file may be omitted, if you do not wish to store that data.
///
/// If you only need a bare IS, start with `MwISBuilder::new()` instead.
pub struct MwFileBuilder<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    writer: W,
}
pub struct MwFileBuilderAwaitsIS {
    /// to prevent users from constructing the struct directly
    _a: (),
}
pub struct MwFileBuilderReadyFrames<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    writer: W,
    off_frames_start: u64,
    frames_hasher: SeaHasher,
    is_hash: u64,
    is_header: ISHeader,
}
pub struct MwFileBuilderReadyCompressFrames<'b, 's, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    writer: W,
    scratch: Cursor<&'s mut Vec<u8>>,
    is_hash: u64,
    is_header: ISHeader,
}
pub struct MwFileBuilderAwaitsFrames {
    off_frames_start: u64,
    is_hash: u64,
}
pub struct MwFileBuilderAwaitsCompressFrames<W: Write + Seek> {
    writer: W,
    is_hash: u64,
}

/// Builder for an Initialization Sequence
///
/// If you only want to create the initialization sequence, such as
/// when a server is starting a new game session and needs to send
/// it over the network, start with `MwISBuilder::new()`.
///
/// If you are creating a full MineWars file, start with `MwFileBuilder::new()`
/// and obtain this via `file_builder.start_is()`.
pub struct MwISBuilder<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    off_header: u32,
    header: ISHeader,
    hasher: Option<SeaHasher>,
    writer: W,
}
pub struct MwISBuilderWithMap<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    off_header: u32,
    header: ISHeader,
    hasher: Option<SeaHasher>,
    writer: W,
}
pub struct MwISBuilderWithCits<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    off_header: u32,
    header: ISHeader,
    hasher: Option<SeaHasher>,
    writer: W,
}
pub struct MwISBuilderWithRules<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    off_header: u32,
    header: ISHeader,
    hasher: Option<SeaHasher>,
    writer: W,
}
pub struct MwISComplete<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    header: ISHeader,
    hash: Option<u64>,
    writer: W,
}

pub struct MwFrameBuilder<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    hasher: Option<SeaHasher>,
    writer: W,
    is_header: ISHeader,
}
pub struct MwFramesComplete<'b, W: Write + Seek> {
    buf: &'b mut Vec<u8>,
    hasher: Option<SeaHasher>,
    writer: W,
    is_header: ISHeader,
}

impl<'b, W: Write + Seek> MwFileBuilder<'b, W> {
    pub fn new(mut writer: W, buf: &'b mut Vec<u8>) -> Result<Self, MwWriterError> {
        writer.seek(SeekFrom::Start(0))?;
        Ok(Self {
            buf,
            writer,
        })
    }
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn start_is(mut self) -> Result<(MwFileBuilderAwaitsIS, MwISBuilder<'b, W>), MwWriterError> {
        // leave space for the file header and IS header
        // to be written at the end when finish is called
        self.writer.seek(SeekFrom::Start(
            MwFileHeader::serialized_len() as u64 +
            ISHeader::serialized_len() as u64
        ))?;
        Ok((
            MwFileBuilderAwaitsIS {
                _a: (),
            },
            MwISBuilder {
                buf: self.buf,
                off_header: MwFileHeader::serialized_len() as u32,
                header: Default::default(),
                hasher: Some(SeaHasher::default()),
                writer: self.writer,
            },
        ))
    }
}
impl MwFileBuilderAwaitsIS {
    pub fn with_is<'b, W: Write + Seek>(
        self,
        mut is: MwISComplete<'b, W>,
    ) -> Result<MwFileBuilderReadyFrames<'b, W>, MwWriterError> {
        Ok(MwFileBuilderReadyFrames {
            buf: is.buf,
            off_frames_start: is.writer.stream_position()?,
            writer: is.writer,
            frames_hasher: SeaHasher::default(),
            is_header: is.header,
            is_hash: is.hash.expect("File builder wants IS built with its MwISBuilder!"),
        })
    }
    pub fn with_is_and_frame_compression<'b, 's, W: Write + Seek>(
        self,
        is: MwISComplete<'b, W>,
        scratch: &'s mut Vec<u8>,
    ) -> Result<MwFileBuilderReadyCompressFrames<'b, 's, W>, MwWriterError> {
        Ok(MwFileBuilderReadyCompressFrames {
            buf: is.buf,
            writer: is.writer,
            scratch: Cursor::new(scratch),
            is_header: is.header,
            is_hash: is.hash.expect("File builder wants IS built with its MwISBuilder!"),
        })
    }
}
impl<'b, W: Write + Seek> MwFileBuilderReadyFrames<'b, W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn finish(mut self) -> Result<(), MwWriterError> {
        let mut header = MwFileHeader::default();
        header.checksum_is = self.is_hash;
        header.checksum_framedata = self.frames_hasher.finish();
        header.len_framedata_raw = (self.writer.stream_position()? - self.off_frames_start) as u32; // FIXME overflow
        header.len_framedata_compressed = header.len_framedata_raw;
        let hash_header = {
            let skip = MwFileHeader::checksummable_start_offset();
            self.buf.clear();
            header.serialize(self.buf);
            self.is_header.serialize(self.buf);
            seahash::hash(&self.buf[skip..])
        };
        header.checksum_header = hash_header;
        self.buf.clear();
        header.serialize(self.buf);
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(self.buf)?;

        self.buf.clear();
        Ok(())
    }
    pub fn start_frames(self) -> Result<(MwFileBuilderAwaitsFrames, MwFrameBuilder<'b, W>), MwWriterError> {
        Ok((
            MwFileBuilderAwaitsFrames {
                off_frames_start: self.off_frames_start,
                is_hash: self.is_hash,
            },
            MwFrameBuilder {
                buf: self.buf,
                writer: self.writer,
                is_header: self.is_header,
                hasher: Some(self.frames_hasher),
            },
        ))
    }
}
impl MwFileBuilderAwaitsFrames {
    pub fn with_frames<'b, W: Write + Seek>(self, frames: MwFramesComplete<'b, W>) -> Result<MwFileBuilderReadyFrames<'b, W>, MwWriterError> {
        Ok(MwFileBuilderReadyFrames {
            buf: frames.buf,
            off_frames_start: self.off_frames_start,
            writer: frames.writer,
            frames_hasher: frames.hasher.expect("File builder wants Frames built with its MwFrameBuilder!"),
            is_hash: self.is_hash,
            is_header: frames.is_header,
        })
    }
}
impl<'b, 's, W: Write + Seek> MwFileBuilderReadyCompressFrames<'b, 's, W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn finish(mut self) -> Result<(), MwWriterError> {
        let mut header = MwFileHeader::default();
        self.buf.clear();
        let scratch = self.scratch.into_inner();
        self.buf.resize(lz4_flex::block::get_maximum_output_size(scratch.len()), 0);
        let compr_len = lz4_flex::block::compress_into(&scratch, self.buf)
            .expect("LZ4 compression bug");
        self.buf.truncate(compr_len);
        self.writer.write_all(self.buf)?;

        header.checksum_framedata = seahash::hash(self.buf);
        header.len_framedata_raw = scratch.len() as u32;
        header.len_framedata_compressed = compr_len as u32;
        header.checksum_is = self.is_hash;
        let hash_header = {
            let mut hasher = SeaHasher::default();
            let skip = MwFileHeader::checksummable_start_offset();
            self.buf.clear();
            header.serialize(self.buf);
            hasher.write(&self.buf[skip..]);
            self.buf.clear();
            self.is_header.serialize(self.buf);
            hasher.write(self.buf);
            hasher.finish()
        };
        header.checksum_header = hash_header;

        self.buf.clear();
        header.serialize(self.buf);
        self.writer.seek(SeekFrom::Start(0))?;
        self.writer.write_all(self.buf)?;

        self.buf.clear();
        Ok(())
    }
    pub fn start_frames(self) -> Result<(MwFileBuilderAwaitsCompressFrames<W>, MwFrameBuilder<'b, Cursor<&'s mut Vec<u8>>>), MwWriterError> {
        Ok((
            MwFileBuilderAwaitsCompressFrames {
                writer: self.writer,
                is_hash: self.is_hash,
            },
            MwFrameBuilder {
                buf: self.buf,
                writer: self.scratch,
                hasher: None,
                is_header: self.is_header,
            },
        ))
    }
}
impl<W: Write + Seek> MwFileBuilderAwaitsCompressFrames<W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn with_frames<'b, 's>(self, frames: MwFramesComplete<'b, Cursor<&'s mut Vec<u8>>>) -> Result<MwFileBuilderReadyCompressFrames<'b, 's, W>, MwWriterError> {
        Ok(MwFileBuilderReadyCompressFrames {
            buf: frames.buf,
            writer: self.writer,
            scratch: frames.writer,
            is_hash: self.is_hash,
            is_header: frames.is_header,
        })
    }
}
impl<'b, W: Write + Seek> MwISBuilder<'b, W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn new(mut writer: W, buf: &'b mut Vec<u8>) -> Result<Self, MwWriterError> {
        // remember the start position
        let off_header = writer.stream_position()? as u32;
        // leave space for the IS header
        // to be written at the end when finish is called
        writer.seek(SeekFrom::Current(
            ISHeader::serialized_len() as i64
        ))?;
        Ok(Self {
            buf,
            off_header,
            header: Default::default(),
            writer,
            hasher: None,
        })
    }
    pub fn finish(mut self) -> Result<MwISComplete<'b, W>, MwWriterError> {
        self.buf.clear();
        self.header.serialize(self.buf);
        self.writer.seek(SeekFrom::Start(self.off_header as u64))?;
        self.writer.write_all(self.buf)?;
        Ok(MwISComplete {
            buf: self.buf,
            writer: self.writer,
            header: self.header,
            hash: self.hasher.map(|h| h.finish()),
        })
    }
    pub fn with_map_uncompressed<C: Coord, D: MapTileDataOut, L: MapDataLayout<C>>(
        mut self,
        mapdata: &MapData<C, D, L>,
        include_items: bool,
    ) -> Result<MwISBuilderWithMap<'b, W>, MwWriterError> {
        self.header.map_size = mapdata.size();
        self.header.set_map_topology(C::TOPOLOGY);
        self.buf.clear();
        crate::map::serialize_map_uncompressed(mapdata, include_items, self.buf);
        self.header.len_mapdata_compressed = self.buf.len() as u32;
        if let Some(ref mut h) = &mut self.hasher {
            h.write(self.buf);
        }
        self.writer.write_all(self.buf)?;
        Ok(MwISBuilderWithMap {
            buf: self.buf,
            off_header: self.off_header,
            header: self.header,
            writer: self.writer,
            hasher: self.hasher,
        })
    }
    pub fn with_map_lz4compressed<C: Coord, D: MapTileDataOut, L: MapDataLayout<C>>(
        mut self,
        mapdata: &MapData<C, D, L>,
        include_items: bool,
        scratch: &mut Vec<u8>,
    ) -> Result<MwISBuilderWithMap<'b, W>, MwWriterError> {
        self.header.map_size = mapdata.size();
        self.header.set_map_topology(C::TOPOLOGY);
        self.buf.clear();
        crate::map::serialize_map_lz4compressed(mapdata, include_items, self.buf, scratch);
        self.header.len_mapdata_compressed = self.buf.len() as u32;
        if let Some(ref mut h) = &mut self.hasher {
            h.write(self.buf);
        }
        self.writer.write_all(self.buf)?;
        Ok(MwISBuilderWithMap {
            buf: self.buf,
            off_header: self.off_header,
            header: self.header,
            writer: self.writer,
            hasher: self.hasher,
        })
    }
}
impl<'b, W: Write + Seek> MwISBuilderWithMap<'b, W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn finish(mut self) -> Result<MwISComplete<'b, W>, MwWriterError> {
        self.buf.clear();
        self.header.serialize(self.buf);
        self.writer.seek(SeekFrom::Start(self.off_header as u64))?;
        self.writer.write_all(self.buf)?;
        Ok(MwISComplete {
            buf: self.buf,
            writer: self.writer,
            header: self.header,
            hash: self.hasher.map(|h| h.finish()),
        })
    }
    pub fn with_cits<'a>(mut self, cit_locations: impl IntoIterator<Item = (Pos, &'a [Ph])>) -> Result<MwISBuilderWithCits<'b, W>, MwWriterError> {
        self.buf.clear();
        self.buf.resize(2 * 256, 0);
        let mut count = 0;
        for (pos, name) in cit_locations {
            if count == 255 {
                break;
            }
            self.buf[count * 2 + 0] = pos.y() as u8;
            self.buf[count * 2 + 1] = pos.x() as u8;
            if name.len() >= 256 {
                self.buf.push(0);
            } else {
                let name_bytes: &[u8] = unsafe {
                    std::mem::transmute(name)
                };
                self.buf.push(name_bytes.len() as u8);
                self.buf.extend_from_slice(name_bytes);
            }
            count += 1;
        }
        self.header.n_regions = count as u8;
        {
            let b_pos = &self.buf[..(count * 2)];
            let b_names = &self.buf[(2 * 256)..];
            self.header.len_citdata_names = b_names.len() as u16;
            if let Some(ref mut h) = &mut self.hasher {
                h.write(b_pos);
                h.write(b_names);
            }
            self.writer.write_all(b_pos)?;
            self.writer.write_all(b_names)?;
        }
        Ok(MwISBuilderWithCits {
            buf: self.buf,
            off_header: self.off_header,
            header: self.header,
            writer: self.writer,
            hasher: self.hasher,
        })
    }
}
impl<'b, W: Write + Seek> MwISBuilderWithCits<'b, W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn finish(mut self) -> Result<MwISComplete<'b, W>, MwWriterError> {
        self.buf.clear();
        self.header.serialize(self.buf);
        self.writer.seek(SeekFrom::Start(self.off_header as u64))?;
        self.writer.write_all(self.buf)?;
        Ok(MwISComplete {
            buf: self.buf,
            writer: self.writer,
            header: self.header,
            hash: self.hasher.map(|h| h.finish()),
        })
    }
    pub fn with_rules(self) -> Result<MwISBuilderWithRules<'b, W>, MwWriterError> {
        Ok(MwISBuilderWithRules {
            buf: self.buf,
            off_header: self.off_header,
            header: self.header,
            writer: self.writer,
            hasher: self.hasher,
        })
    }
}
impl<'b, W: Write + Seek> MwISBuilderWithRules<'b, W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn finish(mut self) -> Result<MwISComplete<'b, W>, MwWriterError> {
        self.buf.clear();
        self.header.serialize(self.buf);
        self.writer.seek(SeekFrom::Start(self.off_header as u64))?;
        self.writer.write_all(self.buf)?;
        Ok(MwISComplete {
            buf: self.buf,
            writer: self.writer,
            header: self.header,
            hash: self.hasher.map(|h| h.finish()),
        })
    }
}
impl<'b, W: Write + Seek> MwFrameBuilder<'b, W> {
    pub fn into_inner(self) -> W {
        self.writer
    }
    pub fn finish(self) -> Result<MwFramesComplete<'b, W>, MwWriterError> {
        Ok(MwFramesComplete {
            buf: self.buf,
            writer: self.writer,
            hasher: self.hasher,
            is_header: self.is_header,
        })
    }
    pub fn append_msgs(&mut self) -> Result<(), MwWriterError> {
        todo!()
    }
    pub fn append_raw_data(&mut self, raw_data: &[u8]) -> Result<(), MwWriterError> {
        if let Some(ref mut h) = &mut self.hasher {
            h.write(raw_data);
        }
        self.writer.write_all(raw_data)?;
        Ok(())
    }
}
