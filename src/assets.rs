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
        app.add_dynamic_collection_to_loading_state::<_, StandardDynamicAssetCollection>(
            AppState::AssetsLoading,
            "locale.assets.ron",
        );
        app.add_collection_to_loading_state::<_, UiAssets>(AppState::AssetsLoading);
        app.add_collection_to_loading_state::<_, SplashAssets>(AppState::AssetsLoading);
        app.add_collection_to_loading_state::<_, TitleLogo>(AppState::AssetsLoading);
        app.add_collection_to_loading_state::<_, GameAssets>(AppState::AssetsLoading);
        app.add_collection_to_loading_state::<_, LocaleAssets>(AppState::AssetsLoading);
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
    #[asset(key = "ui.font2.light")]
    pub font2_light: Handle<Font>,
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
pub struct TitleLogo {
    #[asset(key = "ui.img.logo")]
    pub image: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(key = "game.tilemap.sprites")]
    pub sprites: Handle<TextureAtlas>,
    #[asset(key = "game.tilemap.roads6")]
    pub roads6: Handle<TextureAtlas>,
    #[asset(key = "game.tilemap.roads4")]
    pub roads4: Handle<TextureAtlas>,
}

#[derive(AssetCollection, Resource)]
pub struct LocaleAssets {
    #[asset(key = "locale.bundles", collection(typed))]
    pub bundles: Vec<Handle<bevy_fluent::BundleAsset>>,
}
