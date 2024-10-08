#![allow(unused_variables)]

/// Convenience, to be imported in every file in the crate
/// (and in proprietary)
pub mod prelude {
    pub use mw_app_core::prelude::*;
    pub use bevy_asset_loader::prelude::*;
}

use mw_app_core::TokioRuntime;
use mw_engine::settings_manager::SettingsStore;
use settings::EngineSetupSettings;

use crate::prelude::*;

mod settings;
mod user;

mod camera;
mod haptic;
mod input;
mod map;
mod cit;
mod player;
mod view;

pub mod ui;

mod splash;

mod cli;

#[cfg(feature = "dev")]
mod dev;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::camera::plugin,
        crate::cli::plugin,
        crate::haptic::plugin,
        crate::input::plugin,
        crate::map::plugin,
        crate::cit::plugin,
        crate::player::plugin,
        crate::settings::plugin,
        crate::splash::plugin,
        crate::ui::plugin,
        crate::user::plugin,
        crate::view::plugin,
    ));
    #[cfg(feature = "dev")]
    app.add_plugins((
        crate::dev::plugin,
    ));
}

pub fn setup_bevy_app() -> App {
    let mut app = App::new();
    crate::settings::register_engine_settings(&mut app);
    mw_engine::settings_manager::early_load_settings(
        &mut app, &[SETTINGS_ENGINE]
    );
    app.insert_resource(ClearColor(Color::BLACK));
    let setup_settings = app.world().resource::<SettingsStore>()
        .get::<EngineSetupSettings>().cloned().unwrap();
    let tokio_rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(setup_settings.cpu_threads_net)
        .thread_name("minewars-net")
        .enable_all()
        .build()
        .expect("Could not set up tokio runtime.");
    app.insert_resource(TokioRuntime(tokio_rt));
    let bevy_plugins = DefaultPlugins;
    let bevy_plugins = bevy_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MineWars™ PRE-ALPHA".into(),
            resizable: true,
            ..Default::default()
        }),
        ..Default::default()
    });
    #[cfg(feature = "dev")]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=trace,mw_app_core=trace,mw_app_io=trace,mw_app=trace".into(),
        level: bevy::log::Level::TRACE,
        ..default()
    });
    #[cfg(not(feature = "dev"))]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
        level: bevy::log::Level::INFO,
        ..default()
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
        task_pool_options: TaskPoolOptions::from(&setup_settings),
    });
    app.add_plugins(
        if setup_settings.pipelined_rendering {
            bevy_plugins
        } else {
            bevy_plugins.build()
                .disable::<bevy::render::pipelined_rendering::PipelinedRenderingPlugin>()
        }
    );
    app.add_plugins((
        bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        bevy::diagnostic::EntityCountDiagnosticsPlugin,
        bevy::diagnostic::SystemInformationDiagnosticsPlugin,
    ));
    app
}
