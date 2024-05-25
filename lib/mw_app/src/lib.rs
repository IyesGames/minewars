#![allow(unused_variables)]

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
    pub use mw_app_core::prelude::*;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

use mw_app_core::settings::SettingsStore;
use settings::EngineSetupSettings;

use crate::prelude::*;

mod settings;
mod net;
mod user;

mod camera;
mod haptic;
mod input;
mod map;
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
        crate::net::plugin,
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
    mw_app_core::settings::early_load_settings(
        &mut app, &[SETTINGS_ENGINE]
    );
    app.insert_resource(ClearColor(Color::BLACK));
    let setup_settings = app.world.resource::<SettingsStore>()
        .get::<EngineSetupSettings>().cloned().unwrap();
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
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=trace,mw_app_core=trace,mw_app=trace".into(),
        level: bevy::log::Level::TRACE,
        update_subscriber: None,
    });
    #[cfg(not(feature = "dev"))]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn".into(),
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
