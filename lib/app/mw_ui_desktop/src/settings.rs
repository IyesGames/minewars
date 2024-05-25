use mw_app_core::value::Lch;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<DesktopUiSettings>(SETTINGS_USER.as_ref());
}

/// General UI Settings
#[derive(Reflect, Clone, PartialEq)]
#[reflect(Setting)]
pub struct DesktopUiSettings {
    pub text_scale: f32,
    pub underscan_ratio: f32,
    pub ultrawide_use_extra_width_ratio: f32,
    pub color_text: Lch,
    pub color_text_inactive: Lch,
    pub color_menu_button: Lch,
    pub color_menu_button_inactive: Lch,
    pub color_menu_button_selected: Lch,
}

impl Default for DesktopUiSettings {
    fn default() -> Self {
        DesktopUiSettings {
            text_scale: 1.0,
            underscan_ratio: 1.0,
            ultrawide_use_extra_width_ratio: 0.0,
            color_text: Lch(0.96, 0.125, 80.0),
            color_text_inactive: Lch(0.9, 0.125, 80.0),
            color_menu_button: Lch(0.25, 0.125, 280.0),
            color_menu_button_inactive: Lch(0.125, 0.125, 20.0),
            color_menu_button_selected: Lch(0.2, 0.2, 280.0),
        }
    }
}

impl Setting for DesktopUiSettings {}
