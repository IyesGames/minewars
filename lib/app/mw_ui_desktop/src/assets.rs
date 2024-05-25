use bevy_asset_loader::prelude::*;
use mw_app_core::assets::properties::{NinePatchMargins, PropertiesDynamicAssetCollection};

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
}
