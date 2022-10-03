use crate::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerPaletteSettings>();
    }
}

/// The color palette to use for different players
///
/// Indexed by player ID (0 = neutral)
pub struct PlayerPaletteSettings {
    pub visible: [Color; 7],
    pub fog: [Color; 7],
    pub pending: [Color; 7],
}

impl Default for PlayerPaletteSettings {
    fn default() -> Self {
        PlayerPaletteSettings {
            visible: [
                Color::rgb_u8(200, 200, 200),
                Color::rgb_u8(255, 96, 96),
                Color::rgb_u8(127, 127, 255),
                Color::rgb_u8(123, 231, 123),
                Color::rgb_u8(233, 212, 0),
                Color::rgb_u8(204, 128, 255),
                Color::rgb_u8(250, 120, 160),
            ],
            pending: [
                Color::rgb_u8(210, 210, 210),
                Color::rgb_u8(255, 120, 120),
                Color::rgb_u8(160, 160, 255),
                Color::rgb_u8(140, 255, 140),
                Color::rgb_u8(240, 220, 80),
                Color::rgb_u8(220, 175, 255),
                Color::rgb_u8(255, 150, 180),
            ],
            fog: [
                Color::rgb_u8(127, 127, 127),
                Color::rgb_u8(160, 42, 42),
                Color::rgb_u8(80, 80, 192),
                Color::rgb_u8(64, 120, 64),
                Color::rgb_u8(140, 130, 20),
                Color::rgb_u8(128, 42, 180),
                Color::rgb_u8(160, 50, 120),
            ],
        }
    }
}
