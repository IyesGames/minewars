use bevy_asset_loader::prelude::*;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(AppState::StartupLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
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
    #[asset(key = "ui.9p.notify_simple")]
    pub img_9p_notify_simple: Handle<Image>,
    #[asset(key = "ui.9p.notify_killfeed")]
    pub img_9p_notify_killfeed: Handle<Image>,
}
