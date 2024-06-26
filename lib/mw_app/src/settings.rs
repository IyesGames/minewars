use mw_game_minesweeper::MinesweeperSettings;
use ron::ser::PrettyConfig;

use crate::{haptic::HapticEventKind, input::{AnalogInput, InputAction}, prelude::*, tool::Tool};

pub fn plugin(app: &mut App) {
    app.configure_stage_set(
        Update,
        SettingsSyncSS,
        resource_exists_and_changed::<AllSettings>
    );
    app.configure_sets(Update,
        SetStage::Want(SettingsSyncSS)
            .run_if(resource_exists::<AllSettings>)
    );
    app.add_systems(Update,
        load_or_init_settings
            .run_if(not(resource_exists::<SettingsLoaded>)));
    app.add_systems(Update, (
        loadscreen_wait_settings.track_progress().run_if(in_state(AppState::AssetsLoading)),
    ));
}

#[derive(Resource)]
pub struct SettingsLoaded;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SettingsSyncSS;

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
    pub game: MinewarsGameSettings,
    pub game_minesweeper: MinesweeperSettings,
    pub input: InputSettings,
}

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq)]
#[derive(Serialize, Deserialize)]
pub enum MwRenderer {
    Sprites,
    #[default]
    Tilemap,
    Simple3D,
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
    pub mouse: MouseSettings,
    pub keyboard: KeyboardSettings,
    pub gamepad: GamepadSettings,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MouseSettings {
    // pub millis_click: u16,
    pub map: HashMap<MouseButton, InputAction>,
    pub scroll: InputAction,
    pub edge_pan: bool,
    pub edge_pan_speed: f32,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct KeyboardSettings {
    pub keymap: HashMap<KeyCode, InputAction>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct GamepadSettings {
    pub gridcursor_nonlinear: bool,
    pub gridcursor_sens: f32,
    pub pan_nonlinear: bool,
    pub pan_sens: f32,
    pub buttonmap: HashMap<GamepadButtonType, InputAction>,
    pub axismap: HashMap<GamepadAxisType, InputAction>,
    pub haptics: HashMap<HapticEventKind, Vec<(f32, f32, f32)>>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct CameraSettings {
    pub zoom_tween_duration_ms: u32,
    pub jump_tween_duration_ms: u32,
    pub screenshake: bool,
    pub shake_2d: HashMap<HapticEventKind, Vec<(f32, f32, f32, f32, f32)>>,
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
    pub land_bias: u8,
}

/// Parameters for hosting MineWars games
///
/// (used to set up playground mode and LAN servers)
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct MinewarsGameSettings {
    pub n_plids: u8,
    pub n_cits: u8,
    pub mine_density: u8,
    pub prob_decoy: u8,
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

impl Default for MinewarsGameSettings {
    fn default() -> Self {
        MinewarsGameSettings {
            n_plids: 2,
            n_cits: 5,
            mine_density: 72,
            prob_decoy: 56,
        }
    }
}

impl Default for NetSettings {
    fn default() -> Self {
        NetSettings {
            enabled: true,
            last_host_addr: "127.0.0.1:13370".parse().unwrap(),
            last_host_sessionid: 0,
            worker: NetWorkerConfig {
                ca_cert: "cfg/cert/root.ca.cert.der".into(),
                host_client_cert: vec!["cfg/cert/hostclient.cert.der".into(), "cfg/cert/apps.ca.cert.der".into(), "cfg/cert/root.ca.cert.der".into()],
                host_client_key: "cfg/cert/hostclient.key.der".into(),
                auth_client_cert: vec!["cfg/cert/authclient.cert.der".into(), "cfg/cert/apps.ca.cert.der".into(), "cfg/cert/root.ca.cert.der".into()],
                auth_client_key: "cfg/cert/authclient.key.der".into(),
            },
        }
    }
}

impl Default for InputSettings {
    fn default() -> Self {
        InputSettings {
            mouse: default(),
            keyboard: default(),
            gamepad: default(),
        }
    }
}

impl Default for KeyboardSettings {
    fn default() -> Self {
        let mut keymap = HashMap::default();
        #[cfg(feature = "dev")]
        keymap.insert(KeyCode::Backslash, InputAction::DevDebug);
        keymap.insert(KeyCode::Backquote, InputAction::OpenDevConsole);
        keymap.insert(KeyCode::Tab, InputAction::CycleToolNext);
        keymap.insert(KeyCode::Enter, InputAction::ConfirmCurrentTool);
        keymap.insert(KeyCode::Backspace, InputAction::CancelCurrentTool);
        keymap.insert(KeyCode::Delete, InputAction::CancelCurrentTool);
        keymap.insert(KeyCode::Space, InputAction::UseCurrentTool);
        keymap.insert(KeyCode::KeyQ, InputAction::SwitchTool(Tool::DeployMine));
        keymap.insert(KeyCode::KeyW, InputAction::SwitchTool(Tool::DeployDecoy));
        keymap.insert(KeyCode::KeyE, InputAction::SwitchTool(Tool::DeployTrap));
        keymap.insert(KeyCode::KeyR, InputAction::SwitchTool(Tool::Smoke));
        keymap.insert(KeyCode::KeyA, InputAction::SwitchTool(Tool::Explore));
        keymap.insert(KeyCode::KeyS, InputAction::SwitchTool(Tool::Flag));
        keymap.insert(KeyCode::KeyD, InputAction::SwitchTool(Tool::Reveal));
        keymap.insert(KeyCode::KeyF, InputAction::SwitchTool(Tool::Strike));
        keymap.insert(KeyCode::KeyG, InputAction::SwitchTool(Tool::Harvest));
        keymap.insert(KeyCode::KeyZ, InputAction::SwitchTool(Tool::RemoveStructure));
        keymap.insert(KeyCode::KeyX, InputAction::SwitchTool(Tool::BuildRoad));
        keymap.insert(KeyCode::KeyC, InputAction::SwitchTool(Tool::BuildBridge));
        keymap.insert(KeyCode::KeyV, InputAction::SwitchTool(Tool::BuildWall));
        keymap.insert(KeyCode::KeyB, InputAction::SwitchTool(Tool::BuildTower));
        keymap.insert(KeyCode::Equal, InputAction::ZoomCamera(1.0));
        keymap.insert(KeyCode::Minus, InputAction::ZoomCamera(-1.0));
        keymap.insert(KeyCode::BracketRight, InputAction::RotateCamera(1.0));
        keymap.insert(KeyCode::BracketLeft, InputAction::RotateCamera(-1.0));
        keymap.insert(KeyCode::ArrowLeft, InputAction::PanCamera(Vec2::NEG_X));
        keymap.insert(KeyCode::ArrowRight, InputAction::PanCamera(Vec2::X));
        keymap.insert(KeyCode::ArrowDown, InputAction::PanCamera(Vec2::NEG_Y));
        keymap.insert(KeyCode::ArrowUp, InputAction::PanCamera(Vec2::Y));
        keymap.insert(KeyCode::ControlLeft, InputAction::Analog(AnalogInput::PanCamera));
        keymap.insert(KeyCode::AltLeft, InputAction::Analog(AnalogInput::RotateCamera));
        KeyboardSettings {
            keymap,
        }
    }
}

impl Default for MouseSettings {
    fn default() -> Self {
        let mut map = HashMap::default();
        map.insert(MouseButton::Other(4), InputAction::ConfirmCurrentTool);
        map.insert(MouseButton::Left, InputAction::UseCurrentTool);
        map.insert(MouseButton::Middle, InputAction::UseTool(Tool::Flag));
        map.insert(MouseButton::Right, InputAction::Analog(AnalogInput::PanCamera));
        MouseSettings {
            map,
            scroll: InputAction::Analog(AnalogInput::ZoomCamera),
            edge_pan: true,
            edge_pan_speed: 4.0,
        }
    }
}

impl Default for GamepadSettings {
    fn default() -> Self {
        let mut buttonmap = HashMap::default();
        buttonmap.insert(GamepadButtonType::West, InputAction::ConfirmCurrentTool);
        buttonmap.insert(GamepadButtonType::East, InputAction::CancelCurrentTool);
        buttonmap.insert(GamepadButtonType::South, InputAction::UseCurrentTool);
        buttonmap.insert(GamepadButtonType::LeftTrigger, InputAction::CycleToolPrev);
        buttonmap.insert(GamepadButtonType::RightTrigger, InputAction::CycleToolNext);
        buttonmap.insert(GamepadButtonType::LeftTrigger2, InputAction::ZoomCamera(1.0));
        buttonmap.insert(GamepadButtonType::RightTrigger2, InputAction::ZoomCamera(-1.0));
        let mut axismap = HashMap::default();
        axismap.insert(GamepadAxisType::LeftStickX, InputAction::Analog(AnalogInput::GridCursorMove));
        axismap.insert(GamepadAxisType::LeftStickY, InputAction::Analog(AnalogInput::GridCursorMove));
        axismap.insert(GamepadAxisType::RightStickX, InputAction::Analog(AnalogInput::PanCamera));
        axismap.insert(GamepadAxisType::RightStickY, InputAction::Analog(AnalogInput::PanCamera));
        let mut haptics = HashMap::default();
        haptics.insert(HapticEventKind::ExplosionMineDeath, vec![
            (1.5, 1.0, 0.0),
            (1.0, 1.0, 1.0),
        ]);
        haptics.insert(HapticEventKind::ExplosionOurTerritory, vec![
            (0.25, 0.5, 0.0),
            (0.125, 0.25, 0.5),
        ]);
        haptics.insert(HapticEventKind::ExplosionForeignTerritory, vec![
            (0.25, 0.25, 0.0),
            (0.125, 0.125, 0.25),
        ]);
        haptics.insert(HapticEventKind::BackgroundTremor, vec![
            (0.125, 0.125, 0.0),
            (0.0625, 0.125, 0.125),
        ]);
        haptics.insert(HapticEventKind::ExplosionMineKill, vec![
            (1.25, 0.5, 0.0),
            (1.0, 0.5, 0.5),
        ]);
        haptics.insert(HapticEventKind::ExplosionSomeoneDied, vec![
            (1.25, 0.25, 0.0),
            (1.0, 0.25, 0.25),
        ]);
        GamepadSettings {
            gridcursor_nonlinear: true,
            gridcursor_sens: 420.0,
            pan_nonlinear: true,
            pan_sens: 920.0,
            buttonmap,
            axismap,
            haptics,
        }
    }
}

impl Default for CameraSettings {
    fn default() -> Self {
        let mut shake_2d = HashMap::default();
        shake_2d.insert(HapticEventKind::ExplosionMineDeath, vec![
            (40.0, 11.0, 0.25, 0.5, 1.0),
            (32.0, 13.0, 0.125, 0.25, 1.0),
            (24.0, 17.0, 0.125, 0.5, 1.5),
            (16.0, 19.0, 0.0625, 0.25, 1.5),
        ]);
        shake_2d.insert(HapticEventKind::ExplosionOurTerritory, vec![
            (5.0, 17.0, 0.0625, 0.125, 0.25),
            (4.0, 19.0, 0.125, 0.0625, 0.25),
            (3.0, 23.0, 0.125, 0.125, 0.25),
        ]);
        shake_2d.insert(HapticEventKind::ExplosionForeignTerritory, vec![
            (3.0, 17.0, 0.0625, 0.125, 0.25),
            (2.0, 19.0, 0.125, 0.0625, 0.25),
            (2.0, 23.0, 0.125, 0.125, 0.25),
        ]);
        shake_2d.insert(HapticEventKind::BackgroundTremor, vec![
            (3.0, 17.0, 0.0625, 0.125, 0.25),
            (2.0, 23.0, 0.125, 0.0625, 0.25),
        ]);
        shake_2d.insert(HapticEventKind::ExplosionMineKill, vec![
            (24.0, 11.0, 0.125, 0.25, 0.5),
            (20.0, 13.0, 0.0625, 0.25, 0.5),
            (16.0, 17.0, 0.125, 0.25, 0.5),
            (12.0, 19.0, 0.0625, 0.25, 0.5),
        ]);
        shake_2d.insert(HapticEventKind::ExplosionSomeoneDied, vec![
            (14.0, 11.0, 0.125, 0.125, 0.5),
            (12.0, 13.0, 0.0625, 0.25, 0.5),
            (8.0, 17.0, 0.125, 0.25, 0.5),
            (6.0, 19.0, 0.0625, 0.25, 0.5),
        ]);
        CameraSettings {
            zoom_tween_duration_ms: 125,
            jump_tween_duration_ms: 125,
            screenshake: true,
            shake_2d,
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
            minimap_scale: 2,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
pub struct Lcha(pub f32, pub f32, pub f32);

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
            land_bias: 64,
            seed: None,
        }
    }
}

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
                let path = dir.join("settings.ron");
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        if let Ok(s) = std::str::from_utf8(&bytes) {
                            match ron::from_str(s) {
                                Ok(loaded) => {
                                    info!("Settings successfully loaded from: {:?}", path);
                                    settings = loaded;
                                },
                                Err(e) => {
                                    error!("Error parsing user prefs from RON: {}", e);
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
            let path = dir.join("settings.ron");
            do_write_settings(&settings, &dir, &path);
        }
    });
}

fn do_write_settings(
    settings: &AllSettings,
    dir: &std::path::Path,
    file: &std::path::Path,
) {
    let bytes = ron::ser::to_string_pretty(&settings, PrettyConfig::new())
        .expect("Settings could not be serialized to ron!");
    if let Err(e) = std::fs::create_dir_all(dir) {
        error!("Failed to create user preferences directory: {}", e);
    }
    if let Err(e) = std::fs::write(file, bytes) {
        error!("Failed to write settings to user prefs file: {}", e);
    }
    info!("Settings written to: {:?}", file);
}

fn loadscreen_wait_settings(
    settings: Option<Res<AllSettings>>,
) -> Progress {
    settings.is_some().into()
}
