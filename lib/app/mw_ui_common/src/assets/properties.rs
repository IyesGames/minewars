use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_asset::<NinePatchMargins>();
}

#[derive(Asset, TypePath, Deserialize, Serialize, Debug, Clone)]
pub struct NinePatchMargins {
    pub max_corner_scale: Option<f32>,
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
    pub center_tile_ratio: Option<f32>,
    pub sides_tile_ratio: Option<f32>,
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
