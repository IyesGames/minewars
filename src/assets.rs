use crate::prelude::*;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(LoadingState::new(AppState::AssetsLoading));
        app.add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            AppState::AssetsLoading,
            "ui.assets.ron",
        );
        app.add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            AppState::AssetsLoading,
            "splash.assets.ron",
        );
        app.add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            AppState::AssetsLoading,
            "game.assets.ron",
        );
        app.add_collection_to_loading_state::<_, UiAssets>(AppState::AssetsLoading);
        app.add_collection_to_loading_state::<_, SplashAssets>(AppState::AssetsLoading);
        app.add_collection_to_loading_state::<_, GameAssets>(AppState::AssetsLoading);
    }
}

#[derive(AssetCollection, Resource)]
pub struct UiAssets {
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
}

#[derive(AssetCollection, Resource)]
pub struct SplashAssets {
    #[asset(key = "splash.iyes.logo")]
    pub iyes_logo: Handle<Image>,
    #[asset(key = "splash.iyes.text")]
    pub iyes_text: Handle<Image>,
    #[asset(key = "splash.bevy")]
    pub bevy: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "game.tilemap.tiles6")]
    pub tiles6: Handle<Image>,
    #[asset(key = "game.tilemap.roads6")]
    pub roads6: Handle<Image>,
    #[asset(key = "game.tilemap.tiles4")]
    pub tiles4: Handle<Image>,
    #[asset(key = "game.tilemap.roads4")]
    pub roads4: Handle<Image>,
    #[asset(key = "game.tilemap.digits")]
    pub digits: Handle<Image>,
    #[asset(key = "game.tilemap.gents")]
    pub gents: Handle<Image>,
    #[asset(key = "game.tilemap.flags")]
    pub flags: Handle<Image>,
}
