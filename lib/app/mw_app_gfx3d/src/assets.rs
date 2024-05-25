use bevy::gltf::Gltf;

use crate::prelude::*;
use bevy_asset_loader::prelude::*;

pub mod ass3d;

pub fn plugin(app: &mut App) {
    app.configure_loading_state(
        LoadingStateConfig::new(AppState::StartupLoading)
            .with_dynamic_assets_file::<StandardDynamicAssetCollection>("gfx3d.assets.ron")
            .load_collection::<Gfx3dAssets>()
    );
    app.add_plugins(
        bevy_common_assets::toml::TomlAssetPlugin::<ass3d::Ass3dConfig>::new(&["ass3d.toml"])
    );
}

#[derive(AssetCollection, Resource)]
pub struct Gfx3dAssets {
    #[asset(key = "gfx3d.fallback_skin")]
    pub fallback_3d: Handle<ass3d::Ass3dConfig>,
    #[asset(key = "gfx3d.fallback_skin.gltf")]
    pub fallback_3d_gltf: Handle<Gltf>,
}
