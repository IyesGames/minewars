use mw_app_core::map::{MapDataOrig, MapDescriptor};

use crate::prelude::*;

pub mod loader;
pub mod saver;

pub mod record;
pub mod replay;

pub fn plugin(app: &mut App) {
    app.init_asset::<MwFile>();
    app.init_asset::<MwMap>();
    app.init_asset::<MwReplay>();
    app.init_asset_loader::<loader::MwFileLoader>();
}

#[derive(Asset, TypePath)]
pub struct MwFile {
    pub map: Handle<MwMap>,
    pub replay: Option<Handle<MwReplay>>,
}

#[derive(Asset, TypePath)]
pub struct MwMap {
    pub topology: Topology,
    pub data: MapDataOrig,
}

#[derive(Asset, TypePath)]
pub struct MwReplay {
    pub map: Handle<MwMap>,
    raw_framedata: Vec<u8>,
}

impl MwMap {
    pub fn descriptor(&self) -> MapDescriptor {
        MapDescriptor {
            size: self.data.map.size(),
            topology: self.topology,
        }
    }
}
