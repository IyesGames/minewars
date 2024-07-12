use bevy::{ecs::system::RunSystemOnce, window::{PresentMode, PrimaryWindow, WindowMode}};
use map_macro::hashbrown::hash_map;
use mw_app_core::input::*;
use mw_common::grid::Topology;

use crate::{input::*, prelude::*};

pub fn plugin(app: &mut App) {
    app.register_type::<Topology>();
    app.register_type::<Vec<KeyCode>>();
    app.register_type::<Vec<MouseButton>>();
    app.register_type::<HashMap<Vec<KeyCode>, String>>();
    app.register_type::<HashMap<Vec<MouseButton>, String>>();
    app.register_type::<HashMap<Vec<KeyCode>, HashMap<Vec<MouseButton>, InputAnalogName>>>();
    app.init_setting::<WindowSettings>(SETTINGS_LOCAL.as_ref());
    app.init_setting::<GameViewSettings>(SETTINGS_USER.as_ref());
    app.init_setting::<KeyboardMouseMappings>(SETTINGS_USER.as_ref());
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
            action_motion_disambiguate_ms: 250,
        }
    }
}

impl Setting for MouseInputSettings {}

#[derive(Component, Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct KeyboardMouseMappings {
    pub key_actions: HashMap<Vec<KeyCode>, String>,
    pub mouse_actions: HashMap<Vec<KeyCode>, HashMap<Vec<MouseButton>, String>>,
    pub mouse_motion: HashMap<Vec<KeyCode>, HashMap<Vec<MouseButton>, String>>,
    pub mouse_scroll: HashMap<Vec<KeyCode>, HashMap<Vec<MouseButton>, String>>,
}

impl Default for KeyboardMouseMappings {
    fn default() -> Self {
        let key_actions = hash_map! {
        };
        let mouse_actions = hash_map! {
            vec![] => hash_map! {
                vec![MouseButton::Middle] => mw_app_core::camera::input::ACTION_CENTER.into(),
            },
        };
        let mouse_motion = hash_map! {
            vec![] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_GRID_CURSOR.into(),
                vec![MouseButton::Middle] => mw_app_core::camera::input::ANALOG_ROTATE.into(),
                vec![MouseButton::Right] => mw_app_core::camera::input::ANALOG_PAN.into(),
            },
            vec![KeyCode::ControlLeft] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_PAN.into(),
            },
            vec![KeyCode::ControlRight] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_PAN.into(),
            },
            vec![KeyCode::AltLeft] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ROTATE.into(),
            },
            vec![KeyCode::AltRight] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ROTATE.into(),
            },
            vec![KeyCode::ShiftLeft] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ZOOM.into(),
            },
            vec![KeyCode::ShiftRight] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ZOOM.into(),
            },
        };
        let mouse_scroll = hash_map! {
            vec![] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ZOOM.into(),
            },
            vec![KeyCode::ControlLeft] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_PAN.into(),
            },
            vec![KeyCode::ControlRight] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_PAN.into(),
            },
            vec![KeyCode::AltLeft] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ROTATE.into(),
            },
            vec![KeyCode::AltRight] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ROTATE.into(),
            },
            vec![KeyCode::ShiftLeft] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ZOOM.into(),
            },
            vec![KeyCode::ShiftRight] => hash_map! {
                vec![] => mw_app_core::camera::input::ANALOG_ZOOM.into(),
            },
        };
        Self {
            key_actions,
            mouse_actions,
            mouse_motion,
            mouse_scroll,
        }
    }
}

impl Setting for KeyboardMouseMappings {
    fn apply(&self, world: &mut World) {
        world.run_system_once(crate::input::deactivate_all);
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
    pub cpu_threads_compute_min: usize,
    pub cpu_threads_compute_max: usize,
    pub cpu_threads_compute_pct: f32,
    pub cpu_threads_async_compute_min: usize,
    pub cpu_threads_async_compute_max: usize,
    pub cpu_threads_async_compute_pct: f32,
    pub cpu_threads_io_min: usize,
    pub cpu_threads_io_max: usize,
    pub cpu_threads_io_pct: f32,
}

impl Default for EngineSetupSettings {
    fn default() -> Self {
        let physical = num_cpus::get_physical();
        let logical = num_cpus::get();
        let compute_require = if physical < 4 {
            logical
        } else {
            physical
        };
        EngineSetupSettings {
            pipelined_rendering: true,
            cpu_threads_compute_min: compute_require,
            cpu_threads_compute_max: logical,
            cpu_threads_compute_pct: 100.0,
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
                percent: s.cpu_threads_io_pct / 100.0,
            },
            async_compute: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: s.cpu_threads_async_compute_min,
                max_threads: s.cpu_threads_async_compute_max,
                percent: s.cpu_threads_async_compute_pct / 100.0,
            },
            compute: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: s.cpu_threads_compute_min,
                max_threads: s.cpu_threads_compute_max,
                percent: s.cpu_threads_compute_pct / 100.0,
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
