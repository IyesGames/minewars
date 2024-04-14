use bevy::gltf::Gltf;

use crate::prelude::*;

use self::ass3d::Ass3dConfig;

pub mod ass3d;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(AppState::AssetsLoading)
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("ui.assets.ron")
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("splash.assets.ron")
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("game.assets.ron")
                .with_dynamic_assets_file::<StandardDynamicAssetCollection>("locale.assets.ron")
                .load_collection::<UiAssets>()
                .load_collection::<SplashAssets>()
                .load_collection::<TitleLogo>()
                .load_collection::<GameAssets>()
                .load_collection::<LocaleAssets>()
        );
        app.add_plugins(
            bevy_common_assets::toml::TomlAssetPlugin::<ass3d::Ass3dConfig>::new(&["ass3d.toml"])
        );
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
    #[asset(key = "gfx3d.fallback_skin")]
    pub fallback_3d: Handle<Ass3dConfig>,
    #[asset(key = "gfx3d.fallback_skin.gltf")]
    pub fallback_3d_gltf: Handle<Gltf>,
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

#[derive(AssetCollection, Resource)]
pub struct LocaleAssets {
    #[asset(key = "locale.bundles", collection(typed))]
    pub bundles: Vec<Handle<bevy_fluent::BundleAsset>>,
}
