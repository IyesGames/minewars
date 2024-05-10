use mw_app_core::serde::Lcha;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

/// General UI Settings
#[derive(Component, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct DesktopUiSettings {
    pub text_scale: f32,
    pub underscan_ratio: f32,
    pub ultrawide_use_extra_width_ratio: f32,
    pub color_text: Lcha,
    pub color_text_inactive: Lcha,
    pub color_menu_button: Lcha,
    pub color_menu_button_inactive: Lcha,
    pub color_menu_button_selected: Lcha,
}

impl Default for DesktopUiSettings {
    fn default() -> Self {
        DesktopUiSettings {
            text_scale: 1.0,
            underscan_ratio: 1.0,
            ultrawide_use_extra_width_ratio: 0.0,
            color_text: Lcha(0.96, 0.125, 80.0),
            color_text_inactive: Lcha(0.9, 0.125, 80.0),
            color_menu_button: Lcha(0.25, 0.125, 280.0),
            color_menu_button_inactive: Lcha(0.125, 0.125, 20.0),
            color_menu_button_selected: Lcha(0.2, 0.2, 280.0),
        }
    }
}
