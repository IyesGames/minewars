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
    pub rotate_snap_threshold: f32,
    pub rotate_hex_snap_interval: f32,
    pub rotate_sq_snap_interval: f32,
    pub scroll_rotate_per_line: f32,
    pub scroll_rotate_per_pixel: f32,
    pub scroll_rotate_allow_fractional_lines: bool,
    pub scroll_rotate_invert_leftside: bool,
}

impl Default for Camera2dInputSettings {
    fn default() -> Self {
        Self {
            edge_pan_margin: 4.0,
            edge_pan_speed: 80.0,
            scroll_pan_per_line: 24.0,
            scroll_pan_per_pixel: 1.0,
            scroll_pan_allow_fractional_lines: true,
            rotate_snap_threshold: 3.0,
            rotate_hex_snap_interval: 30.0,
            rotate_sq_snap_interval: 90.0,
            scroll_rotate_per_line: 3.0,
            scroll_rotate_per_pixel: 0.25,
            scroll_rotate_allow_fractional_lines: true,
            scroll_rotate_invert_leftside: true,
        }
    }
}

impl Setting for Camera2dInputSettings {}
