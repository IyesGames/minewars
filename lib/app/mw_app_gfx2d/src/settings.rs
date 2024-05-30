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
    pub enable_rotate_motion_snapping: bool,
    pub enable_rotate_scroll_pixel_snapping: bool,
    pub enable_rotate_scroll_line_snapping: bool,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub zoom_level_snap_threshold: f32,
    pub scroll_zoom_per_line: f32,
    pub scroll_zoom_per_pixel: f32,
    pub scroll_zoom_allow_fractional_lines: bool,
    pub enable_zoom_motion_snapping: bool,
    pub enable_zoom_scroll_pixel_snapping: bool,
    pub enable_zoom_scroll_line_snapping: bool,
    pub pan_tween_duration: f32,
    pub pan_tween_curve: (Vec2, Vec2),
    pub zoom_tween_duration: f32,
    pub zoom_tween_curve: (Vec2, Vec2),
    pub jump_tween_duration: f32,
    pub jump_tween_curve: (Vec2, Vec2),
}

impl Default for Camera2dControlSettings {
    fn default() -> Self {
        Self {
            edge_pan_margin: 8.0,
            edge_pan_speed: 920.0,
            scroll_pan_per_line: 24.0,
            scroll_pan_per_pixel: 1.0,
            scroll_pan_allow_fractional_lines: true,
            rotate_snap_threshold: 3.0,
            rotate_hex_snap_interval: 30.0,
            rotate_sq_snap_interval: 90.0,
            scroll_rotate_per_line: 1.5,
            scroll_rotate_per_pixel: 0.25,
            scroll_rotate_allow_fractional_lines: true,
            scroll_rotate_invert_leftside: true,
            enable_rotate_motion_snapping: true,
            enable_rotate_scroll_pixel_snapping: true,
            enable_rotate_scroll_line_snapping: true,
            zoom_min: 1.0,
            zoom_max: 8.0,
            zoom_level_snap_threshold: 1.0 + (1.0 / 16.0),
            scroll_zoom_per_line: 1.0 + (1.0 / 8.0),
            scroll_zoom_per_pixel: 1.0 + (1.0 / 64.0),
            scroll_zoom_allow_fractional_lines: false,
            enable_zoom_motion_snapping: true,
            enable_zoom_scroll_pixel_snapping: true,
            enable_zoom_scroll_line_snapping: false,
            jump_tween_duration: 0.250,
            jump_tween_curve: (Vec2::new(0.25, 0.125), Vec2::new(0.25, 1.0)),
            pan_tween_duration: 0.125,
            pan_tween_curve: (Vec2::new(0.25, 0.125), Vec2::new(0.25, 1.0)),
            zoom_tween_duration: 0.125,
            zoom_tween_curve: (Vec2::new(0.25, 0.125), Vec2::new(0.25, 1.0)),
        }
    }
}

impl Setting for Camera2dControlSettings {}
