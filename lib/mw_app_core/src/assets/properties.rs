use crate::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_common_assets::ron::RonAssetPlugin;

pub fn plugin(app: &mut App) {
    app.init_asset::<NinePatchMargins>();
    app.add_plugins((
        RonAssetPlugin::<PropertiesDynamicAssetCollection>::new(&["properties.ron"]),
    ));
    app.configure_loading_state(
        LoadingStateConfig::new(AppState::StartupLoading)
            .register_dynamic_asset_collection::<PropertiesDynamicAssetCollection>()
    );
}

#[derive(Asset, TypePath, Deserialize, Serialize, Debug, Clone)]
pub struct NinePatchMargins {
    max_corner_scale: Option<f32>,
    top: f32,
    bottom: f32,
    left: f32,
    right: f32,
    center_tile_ratio: Option<f32>,
    sides_tile_ratio: Option<f32>,
}

impl From<&NinePatchMargins> for ImageScaleMode {
    fn from(value: &NinePatchMargins) -> Self {
        ImageScaleMode::Sliced(TextureSlicer {
            border: BorderRect {
                top: value.top,
                bottom: value.bottom,
                left: value.left,
                right: value.right,
            },
            max_corner_scale: value.max_corner_scale.unwrap_or(1.0),
            center_scale_mode: value.center_tile_ratio
                .map(|r| SliceScaleMode::Tile { stretch_value: r })
                .unwrap_or(SliceScaleMode::Stretch),
            sides_scale_mode: value.sides_tile_ratio
                .map(|r| SliceScaleMode::Tile { stretch_value: r })
                .unwrap_or(SliceScaleMode::Stretch),
        })
    }
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
