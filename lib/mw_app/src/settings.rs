use bevy::window::{PresentMode, PrimaryWindow, WindowMode};
use mw_app_core::{user::UserProfile, value::Lch};
use mw_common::grid::Topology;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<MapGenSettings>(SETTINGS_APP.as_ref());
    app.init_setting::<WindowSettings>(SETTINGS_LOCAL.as_ref());
    app.init_setting::<UserProfileSettings>(SETTINGS_USER.as_ref());
    app.init_setting::<PlidColorSettings>(SETTINGS_USER.as_ref());
}

pub fn register_engine_settings(app: &mut App) {
    app.init_setting::<EngineSetupSettings>(SETTINGS_ENGINE.as_ref());
}

#[derive(Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct PlidColorSettings {
    pub colors: [Lch; 16],
    pub fog: Lch,
}

impl Default for PlidColorSettings {
    fn default() -> Self {
        PlidColorSettings {
            colors: [
                Lch(0.75, 0.0, 0.0),
                Lch(0.5, 0.5, 0.0/15.0 * 360.0),
                Lch(0.5, 0.5, 11.0/15.0 * 360.0),
                Lch(0.5, 0.5, 6.0/15.0 * 360.0),
                Lch(0.5, 0.5, 3.0/15.0 * 360.0),
                Lch(0.5, 0.5, 13.0/15.0 * 360.0),
                Lch(0.5, 0.5, 8.0/15.0 * 360.0),
                Lch(0.5, 0.5, 2.0/15.0 * 360.0),
                Lch(0.5, 0.5, 12.0/15.0 * 360.0),
                Lch(0.5, 0.5, 4.0/15.0 * 360.0),
                Lch(0.5, 0.5, 14.0/15.0 * 360.0),
                Lch(0.5, 0.5, 7.0/15.0 * 360.0),
                Lch(0.5, 0.5, 1.0/15.0 * 360.0),
                Lch(0.5, 0.5, 9.0/15.0 * 360.0),
                Lch(0.5, 0.5, 5.0/15.0 * 360.0),
                Lch(0.5, 0.5, 10.0/15.0 * 360.0),
            ],
            fog: Lch(0.25, 0.0, 0.0),
        }
    }
}

impl Setting for PlidColorSettings {}

#[derive(Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct MapGenSettings {
    pub topology: Topology,
    pub size: u8,
}

impl Default for MapGenSettings {
    fn default() -> Self {
        MapGenSettings { topology: Topology::Hex, size: 24 }
    }
}

impl Setting for MapGenSettings {}

#[derive(Reflect, Debug, Clone)]
#[reflect(Setting)]
pub struct UserProfileSettings(pub UserProfile);

impl Setting for UserProfileSettings {
    fn apply(&self, world: &mut World) {
        use mw_app_core::user::*;
        let mut q = world.query_filtered::<&mut MyUserProfile, With<UserGovernor>>();
        let mut profile = q.single_mut(world);
        profile.0 = self.0.clone();
    }
}

impl Default for UserProfileSettings {
    fn default() -> Self {
        UserProfileSettings(UserProfile {
            display_name: "New Player".into(),
        })
    }
}

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
