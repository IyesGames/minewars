use crate::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;
use mw_ui_common::assets::properties::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        RonAssetPlugin::<PropertiesDynamicAssetCollection>::new(&["properties.ron"]),
    ));
    app.configure_loading_state(
        LoadingStateConfig::new(AppState::StartupLoading)
            .register_dynamic_asset_collection::<PropertiesDynamicAssetCollection>()
    );
}

#[derive(Deserialize, Debug, Clone)]
enum PropertiesDynamicAsset {
    NinePatch {
        max_corner_scale: Option<f32>,
        top: f32,
        bottom: f32,
        left: f32,
        right: f32,
        center_tile_ratio: Option<f32>,
        sides_tile_ratio: Option<f32>,
    },
}

impl DynamicAsset for PropertiesDynamicAsset {
    fn load(&self, asset_server: &AssetServer) -> Vec<UntypedHandle> {
        match self {
            _ => vec![],
        }
    }
    fn build(&self, world: &mut World) -> Result<DynamicAssetType, anyhow::Error> {
        match self {
            &PropertiesDynamicAsset::NinePatch {
                max_corner_scale,
                top, bottom, left, right,
                center_tile_ratio, sides_tile_ratio,
            } => {
                let h = world.resource_mut::<Assets<NinePatchMargins>>()
                    .add(NinePatchMargins {
                        max_corner_scale,
                        top, bottom, left, right,
                        center_tile_ratio, sides_tile_ratio,
                    }).untyped();
                Ok(DynamicAssetType::Single(h))
            }
        }
    }
}

#[derive(Deserialize, Asset, TypePath)]
pub struct PropertiesDynamicAssetCollection(HashMap<String, PropertiesDynamicAsset>);

impl DynamicAssetCollection for PropertiesDynamicAssetCollection {
    fn register(&self, dynamic_assets: &mut DynamicAssets) {
        for (key, asset) in self.0.iter() {
            dynamic_assets.register_asset(key, Box::new(asset.clone()));
        }
    }
}
