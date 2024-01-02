/// Convenience, to be imported in every file in the crate
mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_asset_loader::prelude::*;
    pub use bevy_ecs_tilemap::prelude::*;
    pub use iyes_bevy_extras::prelude::*;
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use iyes_ui::prelude::*;
    pub use mw_common::prelude::*;
    pub use mw_app::prelude::*;
}

use crate::prelude::*;

mod assets;
mod cli;
mod input;
mod gfx2d;
mod locale;
mod minimap;
mod minesweeper;
mod net;
mod screens {
    pub mod loading;
    pub mod splash;
}
mod settings;
mod ui;

#[cfg(feature = "dev")]
mod dev;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));

    let bevy_plugins = DefaultPlugins;
    let bevy_plugins = bevy_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MineWars™ PRE-ALPHA".into(),
            present_mode: bevy::window::PresentMode::Fifo,
            // present_mode: bevy::window::PresentMode::Immediate,
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
    let bevy_plugins = bevy_plugins.set(bevy::asset::AssetPlugin {
        watch_for_changes: bevy::asset::ChangeWatcher::with_delay(Duration::from_millis(250)),
        ..default()
    });
    #[cfg(feature = "dev")]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=trace,mw_app=trace".into(),
        level: bevy::log::Level::TRACE,
    });
    #[cfg(not(feature = "dev"))]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=info,mw_app=info".into(),
        level: bevy::log::Level::INFO,
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
        bevy_plugins.build()
            .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
    );

    app.add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default());

    app.add_plugins(mw_app::MwCommonPlugin);

    // external plugins
    app.add_plugins((
        TilemapPlugin,
        bevy_tweening::TweeningPlugin,
        bevy_fluent::FluentPlugin,
        ProgressPlugin::new(AppState::AssetsLoading).continue_to(AppState::SplashIyes),
        iyes_ui::UiExtrasPlugin,
    ));

    // our stuff
    app.add_plugins((
        crate::screens::loading::LoadscreenPlugin {
            state: AppState::AssetsLoading,
        },
        crate::screens::splash::SplashesPlugin,
        crate::assets::AssetsPlugin,
        crate::locale::LocalePlugin,
        crate::cli::CliPlugin,
        crate::input::InputPlugin,
        crate::ui::UiPlugin,
        crate::settings::SettingsPlugin,
        crate::gfx2d::Gfx2dPlugin,
        crate::minimap::MinimapPlugin,
        crate::net::NetClientPlugin,
        crate::minesweeper::MinesweeperPlugin,
    ));

    #[cfg(feature = "proprietary")]
    app.add_plugins(mw_proprietary_client::MwProprietaryPlugin);

    #[cfg(feature = "dev")]
    app.add_plugins(crate::dev::DevPlugin);

    app.run();
}
