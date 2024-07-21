use crate::prelude::*;
use bevy_asset_loader::prelude::*;

pub mod properties;

pub fn plugin(app: &mut App) {
    app.add_loading_state(
        LoadingState::new(AppState::StartupLoading)
    );
    app.add_plugins((
        self::properties::plugin,
    ));
    app.configure_loading_state(
        LoadingStateConfig::new(AppState::StartupLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("sprites.assets.ron")
            .load_collection::<SpritesAssets>()
    );
}

#[derive(AssetCollection, Resource)]
pub struct SpritesAssets {
    #[asset(key = "sprites.roads6.image")]
    pub roads6_img: Handle<Image>,
    #[asset(key = "sprites.roads6.layout")]
    pub roads6_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.roads4.image")]
    pub roads4_img: Handle<Image>,
    #[asset(key = "sprites.roads4.layout")]
    pub roads4_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.numbers.image")]
    pub numbers_img: Handle<Image>,
    #[asset(key = "sprites.numbers.layout")]
    pub numbers_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.digits.image")]
    pub digits_img: Handle<Image>,
    #[asset(key = "sprites.digits.layout")]
    pub digits_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.tiles.image")]
    pub tiles_img: Handle<Image>,
    #[asset(key = "sprites.tiles.layout")]
    pub tiles_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.explosions.image")]
    pub explosions_img: Handle<Image>,
    #[asset(key = "sprites.explosions.layout")]
    pub explosions_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.gents.image")]
    pub gents_img: Handle<Image>,
    #[asset(key = "sprites.gents.layout")]
    pub gents_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.ui-icons.image")]
    pub ui_icons_img: Handle<Image>,
    #[asset(key = "sprites.ui-icons.layout")]
    pub ui_icons_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "sprites.flags.image")]
    pub flags_img: Handle<Image>,
    #[asset(key = "sprites.flags.layout")]
    pub flags_layout: Handle<TextureAtlasLayout>,
}
