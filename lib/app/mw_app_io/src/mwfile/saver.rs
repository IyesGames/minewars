use std::io::Cursor;

use bevy::{asset::{saver::AssetSaver, AsyncWriteExt}, tasks::futures_lite::AsyncWrite};
use loader::MwFileLoaderSettings;
use mw_app_core::map::MapTileDataOrig;
use mw_dataformat::write::{MwFileBuilder, MwWriterError};

use crate::prelude::*;

use super::*;

#[derive(Debug, Default)]
pub struct MwFileSaver;

#[derive(Debug, Serialize, Deserialize)]
pub struct MwFileSaverSettings {
    pub save_replay: bool,
    pub save_map_items: bool,
    pub compress_map: bool,
    pub compress_frames: bool,
}

#[derive(Debug, Error)]
pub enum MwFileSaverError {
    #[error("Could not save asset: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not encode MineWars format: {0}")]
    Mw(#[from] MwWriterError),
    #[error("Cannot access MwMap asset")]
    NoMapAsset,
}

impl Default for MwFileSaverSettings {
    fn default() -> Self {
        Self {
            save_replay: true,
            save_map_items: true,
            compress_map: true,
            compress_frames: true,
        }
    }
}

impl AssetSaver for MwFileSaver {
    type Asset = MwFile;
    type Settings = MwFileSaverSettings;
    type OutputLoader = loader::MwFileLoader;
    type Error = MwFileSaverError;

    async fn save<'a>(
        &'a self,
        writer: &'a mut bevy::asset::io::Writer,
        asset: bevy::asset::saver::SavedAsset<'a, Self::Asset>,
        settings: &'a Self::Settings,
    ) -> Result<MwFileLoaderSettings, MwFileSaverError> {
        let mwmap = asset.get_labeled::<MwMap, _>("Map")
            .ok_or(MwFileSaverError::NoMapAsset)?
            .get();
        let mwreplay = asset.get_labeled::<MwReplay, _>("Replay")
            .map(|x| x.get());
        save_mwfile(writer, settings, mwmap, mwreplay).await?;
        let out_settings = loader::MwFileLoaderSettings {
            load_replay: settings.save_replay,
            load_map_items: settings.save_map_items,
            verify_checksums: true,
        };
        Ok(out_settings)
    }
}

pub async fn save_mwfile(
    writer: &mut (dyn AsyncWrite + Unpin + Send + Sync),
    settings: &MwFileSaverSettings,
    mwmap: &MwMap,
    mut mwreplay: Option<&MwReplay>,
) -> Result<(), MwFileSaverError> {
    let mut scratch = Vec::new();
    let mut buf = Vec::new();
    let mut out = Vec::new();
    let (b_file, b_is) = MwFileBuilder::new(Cursor::new(&mut out), &mut buf)?
        .start_is()?;
    let b_is = match mwmap.topology {
        Topology::Hex => {
            let map: MapDataC<Hex, MapTileDataOrig> = mwmap.data.map.clone().rekey();
            if settings.compress_map {
                b_is.with_map_lz4compressed(&map, settings.save_map_items, &mut scratch)?
            } else {
                b_is.with_map_uncompressed(&map, settings.save_map_items)?
            }
        }
        Topology::Sq => {
            let map: MapDataC<Sq, MapTileDataOrig> = mwmap.data.map.clone().rekey();
            if settings.compress_map {
                b_is.with_map_lz4compressed(&map, settings.save_map_items, &mut scratch)?
            } else {
                b_is.with_map_uncompressed(&map, settings.save_map_items)?
            }
        }
    };
    let b_is = b_is.with_cits(mwmap.data.cits.iter().map(|pos| (*pos, [].as_slice())))?;
    // TODO: rules
    // TODO: players
    if !settings.save_replay {
        mwreplay = None;
    };
    if let Some(mwreplay) = mwreplay {
        if settings.compress_frames {
            let b_file = b_file.with_is_and_frame_compression(b_is.finish()?, &mut scratch)?;
            let (b_file, mut b_frames) = b_file.start_frames()?;
            b_frames.append_raw_data(&mwreplay.raw_framedata)?;
            let b_file = b_file.with_frames(b_frames.finish()?)?;
            b_file.finish()?;
        } else {
            let b_file = b_file.with_is(b_is.finish()?)?;
            let (b_file, mut b_frames) = b_file.start_frames()?;
            b_frames.append_raw_data(&mwreplay.raw_framedata)?;
            let b_file = b_file.with_frames(b_frames.finish()?)?;
            b_file.finish()?;
        }
    } else {
        let b_file = b_file.with_is(b_is.finish()?)?;
        b_file.finish()?;
    }
    Ok(writer.write_all(&out).await?)
}
