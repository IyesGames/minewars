/// Convenience, to be imported in every file in the crate
mod prelude {
    pub use crate::PROPRIETARY;
    pub use bevy::prelude::*;
    pub use bevy_asset_loader::prelude::*;
    pub use bevy_ecs_tilemap::prelude::*;
    pub use bevy_kira_audio::prelude::*;
    pub use bevy_prototype_lyon::prelude::*;
    pub use iyes_bevy_extras::prelude::*;
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use iyes_ui::prelude::*;
    pub use mw_common::prelude::*;
}

use crate::prelude::*;

pub const PROPRIETARY: bool = cfg!(feature = "mw_proprietary");

mod assets;
mod cli;
mod locale;
mod screens {
    pub mod loading;
    pub mod splash;
}
mod settings;
mod ui;

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));
    let bevy_plugins = DefaultPlugins;
    let bevy_plugins = bevy_plugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "MineWars™ PRE-ALPHA".into(),
            present_mode: bevy::window::PresentMode::Fifo,
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
        watch_for_changes: true,
        ..default()
    });
    #[cfg(feature = "dev")]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=trace".into(),
        level: bevy::log::Level::TRACE,
    });
    #[cfg(not(feature = "dev"))]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=info".into(),
        level: bevy::log::Level::INFO,
    });
    app.add_plugins(bevy_plugins);
    app.add_plugin(TilemapPlugin);
    app.add_plugin(ShapePlugin);
    app.add_plugin(bevy_tweening::TweeningPlugin);
    app.add_plugin(bevy_kira_audio::AudioPlugin);
    app.add_plugin(bevy_fluent::FluentPlugin);

    app.add_state::<AppState>();
    app.add_plugin(ProgressPlugin::new(AppState::AssetsLoading).continue_to(AppState::SplashIyes));
    #[cfg(feature = "dev")]
    app.add_system(debug_progress.run_if(in_state(AppState::AssetsLoading)));

    app.add_plugin(crate::cli::CliPlugin);
    app.add_plugin(mw_proprietary::MwProprietaryPlugin);

    app.add_plugin(screens::loading::LoadscreenPlugin {
        state: AppState::AssetsLoading,
    });
    app.add_plugin(screens::splash::SplashesPlugin);
    app.add_plugin(crate::assets::AssetsPlugin);
    app.add_plugin(crate::settings::SettingsPlugin);
    app.add_plugin(crate::locale::LocalePlugin);
    app.add_plugin(crate::ui::UiPlugin);

    app.run();
}

#[allow(dead_code)]
fn debug_progress(counter: Res<ProgressCounter>) {
    let progress = counter.progress();
    let progress_full = counter.progress_complete();
    trace!(
        "Progress: {}/{}; Full Progress: {}/{}",
        progress.done,
        progress.total,
        progress_full.done,
        progress_full.total,
    );
}
