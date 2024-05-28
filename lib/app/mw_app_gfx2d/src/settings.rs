use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<(Vec2, Vec2)>();
    app.init_setting::<Gfx2dImpl>(SETTINGS_LOCAL.as_ref());
    app.init_setting::<Camera2dControlSettings>(SETTINGS_USER.as_ref());
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
pub struct Camera2dControlSettings {
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
    pub zoom_to_exact_levels: ZoomExactLevels,
    pub zoom_jump_level_threshold: f32,
    pub scroll_zoom_per_line: f32,
    pub scroll_zoom_per_pixel: f32,
    pub scroll_zoom_allow_fractional_lines: bool,
    pub motion_zoom_per_pixel: f32,
    pub jump_tween_duration: f32,
    pub jump_tween_curve: (Vec2, Vec2),
    pub zoom_tween_duration: f32,
    pub zoom_tween_curve: (Vec2, Vec2),
}

impl Default for Camera2dControlSettings {
    fn default() -> Self {
        Self {
            edge_pan_margin: 8.0,
            edge_pan_speed: 160.0,
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
            zoom_to_exact_levels: ZoomExactLevels::No,
            zoom_jump_level_threshold: 2.0,
            scroll_zoom_per_line: 2.0,
            scroll_zoom_per_pixel: 1.125,
            scroll_zoom_allow_fractional_lines: false,
            motion_zoom_per_pixel: 1.125,
            jump_tween_duration: 0.250,
            jump_tween_curve: (Vec2::new(0.25, 0.125), Vec2::new(0.25, 1.0)),
            zoom_tween_duration: 0.125,
            zoom_tween_curve: (Vec2::new(0.25, 0.125), Vec2::new(0.25, 1.0)),
        }
    }
}

impl Setting for Camera2dControlSettings {}

#[derive(Reflect, Clone, Copy, PartialEq, Eq)]
pub enum ZoomExactLevels {
    No,
    Snap,
    Jump,
    Tween,
}
