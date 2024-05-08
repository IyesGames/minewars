/// Convenience, to be imported in every file in the crate
/// (and in proprietary)
pub mod prelude {
    pub use bevy::utils::{Duration, Instant};
    pub use bevy_asset_loader::prelude::*;
    pub use iyes_bevy_extras::prelude::*;
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use iyes_ui::prelude::*;
    pub use mw_common::prelude::*;
    pub use mw_engine::prelude::*;
    pub use crate::apporg::*;
    pub use crate::settings::AllSettings;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

pub mod apporg;
pub mod assets;
pub mod bevyhost;
pub mod cli;
pub mod locale;
#[cfg(feature = "gfx2d")]
pub mod gfx2d;
#[cfg(feature = "gfx3d")]
pub mod gfx3d;
pub mod camera;
pub mod haptic;
pub mod input;
pub mod map;
pub mod player;
pub mod tool;
pub mod view;
pub mod settings;
pub mod screens {
    pub mod loading;
    pub mod splash;
}
pub mod net;
pub mod ui;
pub mod minimap;
pub mod minesweeper;

#[cfg(feature = "dev")]
pub mod dev;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    // mw_engine
    app.add_plugins(mw_engine::plugin);
    // external plugins
    app.add_plugins((
        #[cfg(feature = "gfx2d_tilemap")]
        bevy_ecs_tilemap::TilemapPlugin,
        bevy_tweening::TweeningPlugin,
        bevy_fluent::FluentPlugin,
        ProgressPlugin::new(AppState::AssetsLoading).continue_to(AppState::SplashIyes),
        iyes_ui::UiExtrasPlugin,
    ));
    app.add_plugins((
        (
            crate::apporg::plugin,
        ),
        (
            crate::assets::plugin,
            crate::settings::plugin,
            crate::locale::plugin,
            crate::net::plugin,
            crate::camera::plugin,
        ),
        (
            crate::tool::plugin,
            crate::input::plugin,
            crate::haptic::plugin,
        ),
        (
            crate::map::plugin,
            crate::view::plugin,
            crate::minimap::plugin,
            #[cfg(feature = "gfx2d")]
            crate::gfx2d::plugin,
            #[cfg(feature = "gfx3d")]
            crate::gfx3d::plugin,
            crate::ui::plugin,
        ),
        crate::minesweeper::plugin,
        crate::screens::loading::LoadscreenPlugin {
            state: AppState::AssetsLoading,
        },
        crate::screens::splash::plugin,
        crate::cli::plugin,
    ));
    #[cfg(feature = "dev")]
    app.add_plugins(crate::dev::plugin);
}

pub fn setup_bevy_app() -> App {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));
    let bevy_plugins = DefaultPlugins;
    let bevy_plugins = bevy_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MineWars™ PRE-ALPHA".into(),
            // present_mode: bevy::window::PresentMode::Fifo,
            present_mode: bevy::window::PresentMode::AutoNoVsync,
            // mode: bevy::window::WindowMode::Fullscreen,
            resizable: true,
            resolution: bevy::window::WindowResolution::new(800.0, 600.0),
            resize_constraints: WindowResizeConstraints {
                min_width: 800.0,
                min_height: 600.0,
                max_width: f32::INFINITY,
                max_height: f32::INFINITY,
            },
            // scale_factor_override: Some(1.0),
            ..Default::default()
        }),
        ..Default::default()
    });
    #[cfg(feature = "dev")]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=trace,mw_app=trace".into(),
        level: bevy::log::Level::TRACE,
        update_subscriber: None,
    });
    #[cfg(not(feature = "dev"))]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=info,mw_app=info".into(),
        level: bevy::log::Level::INFO,
        update_subscriber: None,
    });
    let compute_threads = {
        let physical = num_cpus::get_physical();
        let logical = num_cpus::get();
        if physical < 4 {
            logical
        } else {
            physical
        }
    };
    let bevy_plugins = bevy_plugins.set(TaskPoolPlugin {
        task_pool_options: TaskPoolOptions {
            min_total_threads: 1,
            max_total_threads: std::usize::MAX,
            io: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: 2,
                max_threads: 4,
                percent: 0.25,
            },
            async_compute: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: 2,
                max_threads: 4,
                percent: 0.25,
            },
            compute: bevy::core::TaskPoolThreadAssignmentPolicy {
                min_threads: compute_threads,
                max_threads: std::usize::MAX,
                percent: 1.0,
            },
        }
    });
    app.add_plugins(
        // bevy_plugins.build()
        //     .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
        bevy_plugins
    );
    app.add_plugins((
        bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        bevy::diagnostic::EntityCountDiagnosticsPlugin,
        bevy::diagnostic::SystemInformationDiagnosticsPlugin,
    ));
    app
}
