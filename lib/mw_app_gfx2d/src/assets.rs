use crate::prelude::*;
use bevy_asset_loader::prelude::*;

pub fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(AppState::StartupLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("gfx2d.assets.ron")
            .load_collection::<Gfx2dAssets>()
    );
}

#[derive(AssetCollection, Resource)]
pub struct Gfx2dAssets {
    #[asset(key = "gfx2d.tilemap.sprites.image")]
    pub sprites_img: Handle<Image>,
    #[asset(key = "gfx2d.tilemap.sprites.layout")]
    pub sprites_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "gfx2d.tilemap.roads6.image")]
    pub roads6_img: Handle<Image>,
    #[asset(key = "gfx2d.tilemap.roads6.layout")]
    pub roads6_layout: Handle<TextureAtlasLayout>,
    #[asset(key = "gfx2d.tilemap.roads4.image")]
    pub roads4_img: Handle<Image>,
    #[asset(key = "gfx2d.tilemap.roads4.layout")]
    pub roads4_layout: Handle<TextureAtlasLayout>,
}
