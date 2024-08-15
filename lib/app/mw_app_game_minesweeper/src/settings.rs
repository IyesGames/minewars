use mw_game_minesweeper::{minegen::MineGenSettings, MinesweeperSettings};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<SimpleMapSettings>(SETTINGS_APP.as_ref());
    app.init_setting::<OfflineMinesweeperSettings>(SETTINGS_APP.as_ref());
}

#[derive(Reflect, Default, Debug, Clone)]
#[reflect(Setting)]
pub struct OfflineMinesweeperSettings {
    pub game: MinesweeperSettings,
    pub minegen: MineGenSettings,
}

impl Setting for OfflineMinesweeperSettings {}

#[derive(Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct SimpleMapSettings {
    pub topology: Topology,
    pub size: u8,
}

impl Default for SimpleMapSettings {
    fn default() -> Self {
        SimpleMapSettings { topology: Topology::Hex, size: 24 }
    }
}

impl Setting for SimpleMapSettings {}
