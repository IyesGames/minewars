use crate::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            load_or_init_settings
                .run_if(not(resource_exists::<SettingsLoaded>())));
    }
}

#[derive(Resource, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct AllSettings {
    pub gameplay: GameplaySettings,
    pub camera: CameraSettings,
    pub ui_hud: UiHudSettings,
    pub player_colors: PlayerPaletteSettings,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CameraSettings {
    pub zoom_tween_duration_ms: u32,
    pub jump_tween_duration_ms: u32,
    pub screenshake: bool,
    pub edge_pan: bool,
    pub edge_pan_speed: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        CameraSettings {
            zoom_tween_duration_ms: 125,
            jump_tween_duration_ms: 125,
            screenshake: true,
            edge_pan: true,
            edge_pan_speed: 4.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GameplaySettings {
    pub show_skulls: bool,
}

impl Default for GameplaySettings {
    fn default() -> Self {
        GameplaySettings {
            show_skulls: true,
        }
    }
}

/// Settings for the in-game UI (HUD)
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UiHudSettings {
    bottom_layout_reverse: bool,
    citylist: bool,
    citylist_show_unowned: bool,
    ultrawide_dead_space_pct: f32,
}

impl Default for UiHudSettings {
    fn default() -> Self {
        UiHudSettings {
            bottom_layout_reverse: false,
            citylist: true,
            citylist_show_unowned: true,
            ultrawide_dead_space_pct: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Lcha(f32, f32, f32);

impl From<Color> for Lcha {
    fn from(value: Color) -> Self {
        let lcha = value.as_lcha_f32();
        Lcha(lcha[0], lcha[1], lcha[2])
    }
}

impl From<Lcha> for Color {
    fn from(value: Lcha) -> Self {
        Color::Lcha {
            lightness: value.0,
            chroma: value.0,
            hue: value.0,
            alpha: 1.0,
        }
    }
}

/// The color palette to use for different players
///
/// Indexed by player ID (0 = neutral)
#[derive(Serialize, Deserialize, Clone)]
pub struct PlayerPaletteSettings {
    pub visible: [Lcha; 8],
    pub fog: Lcha,
    pub pending: [Lcha; 8],
}

impl Default for PlayerPaletteSettings {
    fn default() -> Self {
        PlayerPaletteSettings {
            visible: [
                Lcha(0.75, 0.0, 0.0),
                Lcha(0.75, 0.5, 0.0),
                Lcha(0.75, 0.5, 180.0),
                Lcha(0.75, 0.5, 60.0),
                Lcha(0.75, 0.5, 240.0),
                Lcha(0.75, 0.5, 120.0),
                Lcha(0.75, 0.5, 300.0),
                Lcha(0.75, 0.5, 30.0),
            ],
            pending: [
                Lcha(0.75, 0.0, 0.0),
                Lcha(0.75, 0.25, 0.0),
                Lcha(0.75, 0.25, 180.0),
                Lcha(0.75, 0.25, 60.0),
                Lcha(0.75, 0.25, 240.0),
                Lcha(0.75, 0.25, 120.0),
                Lcha(0.75, 0.25, 300.0),
                Lcha(0.75, 0.25, 30.0),
            ],
            fog: Lcha(0.5, 0.0, 00.0),
        }
    }
}

#[derive(Resource)]
pub struct SettingsLoaded;

fn load_or_init_settings(
    mut commands: Commands,
    mut iothread: Local<Option<std::thread::JoinHandle<AllSettings>>>,
) {
    if let Some(iothr) = iothread.take() {
        if iothr.is_finished() {
            let allsettings = iothr.join().unwrap_or(AllSettings::default());
            commands.insert_resource(allsettings);
            commands.insert_resource(SettingsLoaded);
            info!("Settings: ready.");
        } else {
            *iothread = Some(iothr);
        }
    } else {
        *iothread = Some(std::thread::spawn(move || {
            let dir = directories::ProjectDirs::from(
                "com", "IyesGames", "MineWars",
            ).map(|dirs| dirs.preference_dir().to_owned());

            let mut settings = AllSettings::default();
            if let Some(dir) = dir {
                let path = dir.join("settings.toml");
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        if let Ok(s) = std::str::from_utf8(&bytes) {
                            match toml::from_str(s) {
                                Ok(loaded) => {
                                    info!("Settings successfully loaded from: {:?}", path);
                                    settings = loaded;
                                },
                                Err(e) => {
                                    error!("Error parsing user prefs from TOML: {}", e);
                                    // if there was a problem with the file, early return,
                                    // we don't want to overwrite it
                                    return settings;
                                }
                            }
                        } else {
                            error!("User prefs file is not UTF-8!");
                        }
                    }
                    Err(e) => {
                        error!("Could not read settings from user prefs file: {}", e);
                    }
                }
                // write the settings to the file
                do_write_settings(&settings, &dir, &path);
            }
            settings
        }));
    }
}

pub fn write_settings(
    world: &World,
) {
    let settings = world.resource::<AllSettings>().clone();
    std::thread::spawn(move || {
        let dir = directories::ProjectDirs::from(
            "com", "IyesGames", "MineWars",
        ).map(|dirs| dirs.preference_dir().to_owned());

        if let Some(dir) = dir {
            let path = dir.join("settings.toml");
            do_write_settings(&settings, &dir, &path);
        }
    });
}

fn do_write_settings(
    settings: &AllSettings,
    dir: &std::path::Path,
    file: &std::path::Path,
) {
    let bytes = toml::to_string(&settings)
        .expect("Settings could not be serialized to toml!");
    if let Err(e) = std::fs::create_dir_all(dir) {
        error!("Failed to create user preferences directory: {}", e);
    }
    if let Err(e) = std::fs::write(file, bytes) {
        error!("Failed to write default settings to user prefs file: {}", e);
    }
    info!("Settings written to: {:?}", file);
}

