use bevy::window::{PresentMode, PrimaryWindow, WindowMode};
use mw_app_core::{graphics::GraphicsStyle, input::{InputActionName, InputAnalogName, InputGovernor}, user::UserProfile, value::Lch};
use mw_common::grid::Topology;

use crate::{input::{ActionNameMap, AnalogNameMap, KeyActionMap, KeyAnalogMap, MouseMap}, prelude::*};

pub fn plugin(app: &mut App) {
    app.register_type::<Topology>();
    app.register_type::<HashMap<String, KeyCode>>();
    app.register_type::<HashMap<String, KeyCode>>();
    app.register_type::<HashMap<String, MouseButton>>();
    app.register_type::<HashMap<String, MouseButton>>();
    app.init_setting::<WindowSettings>(SETTINGS_LOCAL.as_ref());
    app.init_setting::<GameViewSettings>(SETTINGS_USER.as_ref());
    app.init_setting::<KeyboardMapSettings>(SETTINGS_USER.as_ref());
    app.init_setting::<MouseMapSettings>(SETTINGS_USER.as_ref());
    app.init_setting::<KeyboardInputSettings>(SETTINGS_USER.as_ref());
    app.init_setting::<MouseInputSettings>(SETTINGS_USER.as_ref());
}

pub fn register_engine_settings(app: &mut App) {
    app.init_setting::<EngineSetupSettings>(SETTINGS_ENGINE.as_ref());
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct KeyboardInputSettings {
}

impl Default for KeyboardInputSettings {
    fn default() -> Self {
        Self {
        }
    }
}

impl Setting for KeyboardInputSettings {}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct MouseInputSettings {
    pub action_motion_disambiguate_ms: u32,
}

impl Default for MouseInputSettings {
    fn default() -> Self {
        Self {
            action_motion_disambiguate_ms: 125,
        }
    }
}

impl Setting for MouseInputSettings {}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct KeyboardMapSettings {
    pub actions: HashMap<String, KeyCode>,
    pub mouse_motion: HashMap<String, KeyCode>,
    pub mouse_scroll: HashMap<String, KeyCode>,
}

impl Default for KeyboardMapSettings {
    fn default() -> Self {
        let mut actions = HashMap::default();
        let mut mouse_motion = HashMap::default();
        let mut mouse_scroll = HashMap::default();
        mouse_motion.insert(
            mw_app_core::camera::input::ANALOG_PAN.into(),
            KeyCode::ControlLeft
        );
        mouse_motion.insert(
            mw_app_core::camera::input::ANALOG_ROTATE.into(),
            KeyCode::AltLeft
        );
        mouse_motion.insert(
            mw_app_core::camera::input::ANALOG_ZOOM.into(),
            KeyCode::ShiftLeft
        );
        Self {
            actions,
            mouse_motion,
            mouse_scroll,
        }
    }
}

impl Setting for KeyboardMapSettings {
    fn apply(&self, world: &mut World) {
        let mut q = world.query_filtered::<(
            &ActionNameMap,
            &AnalogNameMap,
            &mut KeyActionMap,
            &mut KeyAnalogMap,
        ), With<InputGovernor>>();
        let Ok((
            action_name_map,
            analog_name_map,
            mut key_action_map,
            mut key_analog_map,
        )) = q.get_single_mut(world) else {
            return;
        };
        for (name, key) in self.actions.iter() {
            if let Some(e) = action_name_map.map_name.get(name) {
                let key_action_map = &mut *key_action_map;
                if let Some(e) = key_action_map.map_key.get(key) {
                    key_action_map.map_entity.remove(e);
                }
                if let Some(key) = key_action_map.map_entity.get(e) {
                    key_action_map.map_key.remove(key);
                }
                key_action_map.map_key.insert(*key, *e);
                key_action_map.map_entity.insert(*e, *key);
            } else {
                warn!("No Action Input with name {:?} found!", name);
            }
        }
        for (name, key) in self.mouse_motion.iter() {
            if let Some(e) = analog_name_map.map_name.get(name) {
                let key_analog_map = &mut *key_analog_map;
                if let Some(e) = key_analog_map.motion_key.get(key) {
                    key_analog_map.motion_entity.remove(e);
                }
                if let Some(key) = key_analog_map.motion_entity.get(e) {
                    key_analog_map.motion_key.remove(key);
                }
                key_analog_map.motion_key.insert(*key, *e);
                key_analog_map.motion_entity.insert(*e, *key);
            } else {
                warn!("No Analog Input with name {:?} found!", name);
            }
        }
        for (name, key) in self.mouse_scroll.iter() {
            if let Some(e) = analog_name_map.map_name.get(name) {
                let key_analog_map = &mut *key_analog_map;
                if let Some(e) = key_analog_map.scroll_key.get(key) {
                    key_analog_map.scroll_entity.remove(e);
                }
                if let Some(key) = key_analog_map.scroll_entity.get(e) {
                    key_analog_map.scroll_key.remove(key);
                }
                key_analog_map.scroll_key.insert(*key, *e);
                key_analog_map.scroll_entity.insert(*e, *key);
            } else {
                warn!("No Analog Input with name {:?} found!", name);
            }
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct MouseMapSettings {
    pub actions: HashMap<String, MouseButton>,
    pub mouse_motion: HashMap<String, MouseButton>,
}

impl Default for MouseMapSettings {
    fn default() -> Self {
        let mut actions = Default::default();
        let mut mouse_motion = Default::default();
        Self {
            actions,
            mouse_motion,
        }
    }
}

impl Setting for MouseMapSettings {
    fn apply(&self, world: &mut World) {
        let mut q = world.query_filtered::<(
            &ActionNameMap,
            &AnalogNameMap,
            &mut MouseMap,
        ), With<InputGovernor>>();
        let Ok((
            action_name_map,
            analog_name_map,
            mut mouse_map,
        )) = q.get_single_mut(world) else {
            return;
        };
        for (name, btn) in self.actions.iter() {
            if let Some(e) = action_name_map.map_name.get(name) {
                let mouse_map = &mut *mouse_map;
                if let Some(e) = mouse_map.action_btn.get(btn) {
                    mouse_map.action_entity.remove(e);
                }
                if let Some(key) = mouse_map.action_entity.get(e) {
                    mouse_map.action_btn.remove(key);
                }
                mouse_map.action_btn.insert(*btn, *e);
                mouse_map.action_entity.insert(*e, *btn);
            } else {
                warn!("No Action Input with name {:?} found!", name);
            }
        }
        for (name, btn) in self.mouse_motion.iter() {
            if let Some(e) = analog_name_map.map_name.get(name) {
                let mouse_map = &mut *mouse_map;
                if let Some(e) = mouse_map.motion_btn.get(btn) {
                    mouse_map.motion_entity.remove(e);
                }
                if let Some(key) = mouse_map.motion_entity.get(e) {
                    mouse_map.motion_btn.remove(key);
                }
                mouse_map.motion_btn.insert(*btn, *e);
                mouse_map.motion_entity.insert(*e, *btn);
            } else {
                warn!("No Analog Input with name {:?} found!", name);
            }
        }
    }
}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct GameViewSettings {
    pub tile_alert_duration_ms: u32,
}

impl Default for GameViewSettings {
    fn default() -> Self {
        Self {
            tile_alert_duration_ms: 1000,
        }
    }
}

impl Setting for GameViewSettings {}

#[derive(Reflect, Clone, PartialEq)]
#[reflect(Setting)]
pub struct EngineSetupSettings {
    pub pipelined_rendering: bool,
    pub cpu_threads_compute: usize,
    pub cpu_threads_async_compute_min: usize,
    pub cpu_threads_async_compute_max: usize,
    pub cpu_threads_async_compute_pct: f32,
    pub cpu_threads_io_min: usize,
    pub cpu_threads_io_max: usize,
    pub cpu_threads_io_pct: f32,
}

impl Default for EngineSetupSettings {
    fn default() -> Self {
        EngineSetupSettings {
            pipelined_rendering: true,
            cpu_threads_compute: {
                let physical = num_cpus::get_physical();
                let logical = num_cpus::get();
                if physical < 4 {
                    logical
                } else {
                    physical
                }
            },
            cpu_threads_async_compute_min: 2,
            cpu_threads_async_compute_max: 4,
            cpu_threads_async_compute_pct: 25.0,
            cpu_threads_io_min: 2,
            cpu_threads_io_max: 4,
            cpu_threads_io_pct: 25.0,
        }
    }
}

impl From<&EngineSetupSettings> for TaskPoolOptions {
    fn from(s: &EngineSetupSettings) -> Self {
        TaskPoolOptions {
            min_total_threads: 1,
            max_total_threads: std::usize::MAX,
            io: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: s.cpu_threads_io_min,
                max_threads: s.cpu_threads_io_max,
                percent: s.cpu_threads_io_pct,
            },
            async_compute: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: s.cpu_threads_async_compute_min,
                max_threads: s.cpu_threads_async_compute_max,
                percent: s.cpu_threads_async_compute_pct,
            },
            compute: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: s.cpu_threads_compute,
                max_threads: std::usize::MAX,
                percent: 1.0,
            },
        }
    }
}

impl Setting for EngineSetupSettings {}

#[derive(Reflect, Clone, PartialEq)]
#[reflect(Setting)]
pub struct WindowSettings {
    pub resolution: Vec2,
    pub scale_factor_override: Option<f32>,
    pub present_mode: PresentMode,
    pub mode: WindowMode,
}

impl Default for WindowSettings {
    fn default() -> Self {
        WindowSettings {
            resolution: Vec2::new(800.0, 600.0),
            scale_factor_override: None,
            present_mode: PresentMode::AutoNoVsync,
            mode: WindowMode::Windowed,
        }
    }
}

impl Setting for WindowSettings {
    fn apply(&self, world: &mut World) {
        let mut q = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let mut window = q.single_mut(world);
        window.resolution.set_scale_factor_override(self.scale_factor_override);
        window.resolution.set(self.resolution.x, self.resolution.y);
        window.present_mode = self.present_mode;
        window.mode = self.mode;
    }
}
