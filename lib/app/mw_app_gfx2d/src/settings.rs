use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<Gfx2dImpl>(SETTINGS_LOCAL.as_ref());
    app.init_setting::<Camera2dInputSettings>(SETTINGS_USER.as_ref());
}

#[derive(Reflect, Default, Clone, PartialEq, Eq)]
#[reflect(Setting)]
pub enum Gfx2dImpl {
    Sprites,
    #[default]
    Bespoke,
    #[cfg(feature = "tilemap")]
    Tilemap,
}

impl Setting for Gfx2dImpl {}

#[derive(Reflect, Clone, PartialEq)]
#[reflect(Setting)]
pub struct Camera2dInputSettings {
    pub edge_pan_margin: f32,
    pub edge_pan_speed: f32,
    pub scroll_pan_per_line: f32,
    pub scroll_pan_per_pixel: f32,
    pub scroll_pan_allow_fractional_lines: bool,
    pub scroll_pan_invert_x: bool,
    pub scroll_pan_invert_y: bool,
}

impl Default for Camera2dInputSettings {
    fn default() -> Self {
        Self {
            edge_pan_margin: 4.0,
            edge_pan_speed: 80.0,
            scroll_pan_per_line: 24.0,
            scroll_pan_per_pixel: 1.0,
            scroll_pan_allow_fractional_lines: true,
            scroll_pan_invert_x: false,
            scroll_pan_invert_y: false,
        }
    }
}

impl Setting for Camera2dInputSettings {}
