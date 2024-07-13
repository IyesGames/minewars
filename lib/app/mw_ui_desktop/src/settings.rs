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
    pub color_text: Oklcha,
    pub color_text_inactive: Oklcha,
    pub color_menu_button: Oklcha,
    pub color_menu_button_inactive: Oklcha,
    pub color_menu_button_selected: Oklcha,
}

impl Default for DesktopUiSettings {
    fn default() -> Self {
        DesktopUiSettings {
            text_scale: 1.0,
            underscan_ratio: 1.0,
            ultrawide_use_extra_width_ratio: 0.0,
            color_text: Oklcha::new(0.96, 0.125, 80.0, 1.0),
            color_text_inactive: Oklcha::new(0.9, 0.125, 80.0, 1.0),
            color_menu_button: Oklcha::new(0.25, 0.125, 280.0, 1.0),
            color_menu_button_inactive: Oklcha::new(0.125, 0.125, 20.0, 1.0),
            color_menu_button_selected: Oklcha::new(0.2, 0.2, 280.0, 1.0),
        }
    }
}

impl Setting for DesktopUiSettings {}
