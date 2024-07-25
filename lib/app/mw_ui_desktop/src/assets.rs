use bevy_asset_loader::prelude::*;
use mw_app_core::assets::properties::PropertiesDynamicAssetCollection;
use mw_ui_common::assets::properties::NinePatchMargins;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(AppState::StartupLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
            .with_dynamic_assets_file::<PropertiesDynamicAssetCollection>("ui.properties.ron")
            .load_collection::<UiAssets>()
    );
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
    #[asset(key = "ui.img.logo")]
    pub title_logo: Handle<Image>,
    #[asset(key = "ui.font")]
    pub font: Handle<Font>,
    #[asset(key = "ui.font.bold")]
    pub font_bold: Handle<Font>,
    #[asset(key = "ui.font.light")]
    pub font_light: Handle<Font>,
    #[asset(key = "ui.font2")]
    pub font2: Handle<Font>,
    #[asset(key = "ui.font2.bold")]
    pub font2_bold: Handle<Font>,
    #[asset(key = "ui.font2.light")]
    pub font2_light: Handle<Font>,
    #[asset(key = "ui.bg.notify_simple")]
    pub bg_img_notify_simple: Handle<Image>,
    #[asset(key = "ui.bg.notify_simple.9p")]
    pub bg_9p_notify_simple: Handle<NinePatchMargins>,
    #[asset(key = "ui.bg.notify_killfeed")]
    pub bg_img_notify_killfeed: Handle<Image>,
    #[asset(key = "ui.bg.notify_killfeed.9p")]
    pub bg_9p_notify_killfeed: Handle<NinePatchMargins>,
    #[asset(key = "ui.bg.miniboard_icon")]
    pub bg_img_miniboard_icon: Handle<Image>,
    #[asset(key = "ui.bg.miniboard_icon.9p")]
    pub bg_9p_miniboard_icon: Handle<NinePatchMargins>,
}

#[allow(dead_code)]
pub mod sprite {
    pub const TILES6: usize = 0;
    pub const TILES4: usize = 8;
    pub const ICONS: usize = 16;

    pub const TILE_THIN: usize = 0;
    pub const TILE_THICK: usize = 1;
    pub const TILE_SOLID: usize = 2;
    pub const TILE_GRADIENT: usize = 3;
    pub const TILEKIND_WATER: usize = 4;
    pub const TILEKIND_FERTILE: usize = 5;
    pub const TILEKIND_MOUNTAIN: usize = 6;
    pub const TILEKIND_FOREST: usize = 7;

    pub const ICON_X: usize = ICONS + 0;
    pub const ICON_SMOKE: usize = ICONS + 1;
    pub const ICON_STRIKE: usize = ICONS + 2;
    pub const ICON_REVEAL: usize = ICONS + 3;
    pub const ICON_MINE: usize = ICONS + 4;
    pub const ICON_DECOY: usize = ICONS + 5;
    pub const ICON_TRAP: usize = ICONS + 6;
    pub const ICON_MINEACT: usize = ICONS + 7;
    pub const ICON_BRIDGE: usize = ICONS + 8;
    pub const ICON_TOWER: usize = ICONS + 9;
    pub const ICON_WALL: usize = ICONS + 10;
    pub const ICON_ROAD: usize = ICONS + 11;
    pub const ICON_SKULL: usize = ICONS + 15;
}
