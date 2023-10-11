use mw_game_minesweeper::MinesweeperSettings;

use crate::prelude::*;

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(Update, NeedsSettingsSet.run_if(resource_exists::<AllSettings>()));
        app.add_systems(Update,
            load_or_init_settings
                .run_if(not(resource_exists::<SettingsLoaded>())));
        app.add_systems(Update, (
            loadscreen_wait_settings.track_progress().run_if(in_state(AppState::AssetsLoading)),
        ));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeedsSettingsSet;

#[derive(Resource, Serialize, Deserialize, Default, Clone)]
#[serde(default)]
pub struct AllSettings {
    pub renderer: MwRenderer,
    pub gameplay: GameplaySettings,
    pub camera: CameraSettings,
    pub ui: UiSettings,
    pub ui_hud: UiHudSettings,
    pub player_colors: PlayerPaletteSettings,
    pub net: NetSettings,
    pub mapgen: MapGenSettings,
    pub game_minesweeper: MinesweeperSettings,
    pub input: InputSettings,
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum MwRenderer {
    Sprites,
    #[default]
    Tilemap,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct NetSettings {
    pub enabled: bool,
    // TODO: do things via ToSocketAddrs to support DNS
    pub last_host_addr: SocketAddr,
    pub last_host_sessionid: u32,
    pub worker: NetWorkerConfig,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct InputSettings {
    pub millis_click: u16,
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

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GameplaySettings {
    pub show_skulls: bool,
}

/// General UI Settings
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UiSettings {
    pub text_scale: f32,
    pub underscan_ratio: f32,
    pub ultrawide_use_extra_width_ratio: f32,
    pub color_text: Lcha,
    pub color_text_inactive: Lcha,
    pub color_menu_button: Lcha,
    pub color_menu_button_inactive: Lcha,
    pub color_menu_button_selected: Lcha,
}

/// Settings for the in-game UI (HUD)
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct UiHudSettings {
    pub bottom_layout_reverse: bool,
    pub citylist: bool,
    pub citylist_show_unowned: bool,
    pub minimap_scale: u8,
}

/// The color palette to use for different players
///
/// Indexed by player ID (0 = neutral)
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct PlayerPaletteSettings {
    pub flag_style: u8,
    pub visible: [Lcha; 16],
    pub fog: Lcha,
}

/// Parameters for local map generation
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MapGenSettings {
    pub size: u8,
    pub topology: mw_common::grid::Topology,
    pub style: MapGenStyle,
    pub seed: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapGenStyle {
    Flat,
    MineWars,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NetWorkerConfig {
    pub ca_cert: PathBuf,
    pub host_client_cert: Vec<PathBuf>,
    pub host_client_key: PathBuf,
    pub auth_client_cert: Vec<PathBuf>,
    pub auth_client_key: PathBuf,
}

impl Default for NetSettings {
    fn default() -> Self {
        NetSettings {
            enabled: true,
            last_host_addr: "127.0.0.1:13370".parse().unwrap(),
            last_host_sessionid: 0,
            worker: NetWorkerConfig {
                ca_cert: "cert/ca.cert.der".into(),
                host_client_cert: vec!["cert/hostclient.cert.der".into(), "cert/ca.cert.der".into()],
                host_client_key: "cert/hostclient.key.der".into(),
                auth_client_cert: vec!["cert/authclient.cert.der".into(), "cert/ca.cert.der".into()],
                auth_client_key: "cert/authclient.key.der".into(),
            },
        }
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        InputSettings {
            millis_click: 250,
        }
    }
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

impl Default for GameplaySettings {
    fn default() -> Self {
        GameplaySettings {
            show_skulls: true,
        }
    }
}

impl Default for UiSettings {
    fn default() -> Self {
        UiSettings {
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

impl Default for UiHudSettings {
    fn default() -> Self {
        UiHudSettings {
            bottom_layout_reverse: false,
            citylist: true,
            citylist_show_unowned: true,
            minimap_scale: 3,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
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
            chroma: value.1,
            hue: value.2,
            alpha: 1.0,
        }
    }
}

impl Default for PlayerPaletteSettings {
    fn default() -> Self {
        PlayerPaletteSettings {
            flag_style: 0,
            visible: [
                Lcha(0.75, 0.0, 0.0),
                Lcha(0.5, 0.5, 0.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 11.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 6.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 3.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 13.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 8.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 2.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 12.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 4.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 14.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 7.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 1.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 9.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 5.0/15.0 * 360.0),
                Lcha(0.5, 0.5, 10.0/15.0 * 360.0),
            ],
            fog: Lcha(0.25, 0.0, 0.0),
        }
    }
}

impl Default for MapGenSettings {
    fn default() -> Self {
        MapGenSettings {
            size: 24,
            topology: mw_common::grid::Topology::Hex,
            style: if PROPRIETARY {
                MapGenStyle::MineWars
            } else {
                MapGenStyle::Flat
            },
            seed: None,
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

fn loadscreen_wait_settings(
    settings: Option<Res<AllSettings>>,
) -> Progress {
    settings.is_some().into()
}

