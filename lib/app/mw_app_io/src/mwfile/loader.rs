use std::io::Cursor;

use bevy::asset::{io::AsyncReadAndSeek, AssetLoader, AsyncReadExt};
use mw_app_core::map::MapTileDataOrig;
use mw_dataformat::{header::{ISHeader, MwFileHeader}, read::{MwFileReader, MwReaderError}};

use crate::prelude::*;

use super::*;

#[derive(Debug, Default)]
pub struct MwFileLoader;

#[derive(Debug, Serialize, Deserialize)]
pub struct MwFileLoaderSettings {
    pub load_replay: bool,
    pub load_map_items: bool,
    pub verify_checksums: bool,
}

#[derive(Debug, Error)]
pub enum MwFileLoaderError {
    #[error("Could not load asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not decode MineWars format: {0}")]
    Mw(#[from] MwReaderError),
}

impl Default for MwFileLoaderSettings {
    fn default() -> Self {
        Self {
            load_replay: true,
            load_map_items: true,
            verify_checksums: true,
        }
    }
}

impl AssetLoader for MwFileLoader {
    type Asset = MwFile;
    type Settings = MwFileLoaderSettings;
    type Error = MwFileLoaderError;

    fn extensions(&self) -> &[&str] {
        &["minewars"]
    }

    async fn load<'a>(
        &'a self,
        reader: &'a mut bevy::asset::io::Reader<'_>,
        settings: &'a Self::Settings,
        load_context: &'a mut bevy::asset::LoadContext<'_>,
    ) -> Result<MwFile, MwFileLoaderError> {
        let (mwmap, mwreplay) = load_mwfile(reader, settings).await?;
        let h_mwmap = load_context.add_labeled_asset("Map".into(), mwmap);
        let mut mwfile = MwFile {
            map: h_mwmap.clone(),
            replay: None,
        };
        if let Some(mut mwreplay) = mwreplay {
            mwreplay.map = h_mwmap.clone();
            let h_mwreplay = load_context.add_labeled_asset("Replay".into(), mwreplay);
            mwfile.replay = Some(h_mwreplay);
        }
        Ok(mwfile)
    }
}

pub async fn load_mwfile(
    reader: &mut (dyn AsyncReadAndSeek + Unpin + Send + Sync),
    settings: &MwFileLoaderSettings,
) -> Result<(MwMap, Option<MwReplay>), MwFileLoaderError> {
        let mut scratch = Vec::new();
        let mut buf = Vec::new();
        let len_headers = MwFileHeader::serialized_len() + ISHeader::serialized_len();
        let mut bytes = Vec::new();
        let mfr = if settings.load_replay {
            reader.read_to_end(&mut buf).await?;
            let mut mfr = MwFileReader::new(Cursor::new(&mut bytes), &mut buf)?;
            if settings.verify_checksums {
                mfr.verify_checksums()?;
            }
            mfr
        } else {
            bytes.resize(len_headers, 0);
            reader.read_exact(&mut bytes).await?;
            let len_isdata = {
                let mut mfr = MwFileReader::new(Cursor::new(&mut bytes), &mut buf)?;
                if settings.verify_checksums {
                    mfr.verify_checksum_header()?;
                }
                mfr.len_isdata()
            };
            bytes.resize(len_headers + len_isdata, 0);
            reader.read_exact(&mut bytes[len_headers..]).await?;
            let mut mfr = MwFileReader::new(Cursor::new(&mut bytes), &mut buf)?;
            if settings.verify_checksums {
                mfr.verify_checksum_isdata()?;
            }
            mfr
        };
        let (mfr, mut isr) = mfr.read_is()?;
        let map: MapDataPos<MapTileDataOrig> = match isr.map_topology() {
            Topology::Hex => {
                let map: MapDataC<Hex, MapTileDataOrig> =
                    isr.read_map(Some(&mut scratch), settings.load_map_items)?;
                map.rekey()
            }
            Topology::Sq => {
                let map: MapDataC<Sq, MapTileDataOrig> =
                    isr.read_map(Some(&mut scratch), settings.load_map_items)?;
                map.rekey()
            }
        };
        let cits = isr.read_cits_pos()?.to_owned();
        let orig = MapDataOrig {
            map, cits,
        };
        let mwmap = MwMap {
            topology: isr.map_topology(),
            data: orig,
        };
        if settings.load_replay {
            let mut mfr = mfr.finish_is(isr)?;
            let mwreplay = MwReplay {
                map: default(),
                raw_framedata: mfr.get_uncompressed_framedata()?,
            };
            Ok((mwmap, Some(mwreplay)))
        } else {
            Ok((mwmap, None))
        }
}
