/// Convenience, to be imported in every file in the crate
mod prelude {
    pub use bevy::prelude::*;
    pub use bevy_asset_loader::prelude::*;
    pub use bevy_ecs_tilemap::prelude::*;
    pub use bevy_prototype_lyon::prelude::*;
    pub use iyes_bevy_extras::prelude::*;
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use iyes_ui::prelude::*;
    pub use mw_common::prelude::*;
    pub use mw_app::prelude::*;
    pub use mw_proprietary_client::PROPRIETARY;
    pub use crate::settings::AllSettings;
}

use crate::prelude::*;

mod assets;
mod camera;
mod cli;
mod game;
mod gfx2d;
mod input;
mod locale;
mod minimap;
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
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=trace".into(),
        level: bevy::log::Level::TRACE,
    });
    #[cfg(not(feature = "dev"))]
    let bevy_plugins = bevy_plugins.set(bevy::log::LogPlugin {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=info".into(),
        level: bevy::log::Level::INFO,
    });
    app.add_plugins(bevy_plugins);

    app.add_plugins(mw_app::MwCommonPlugin);

    // external plugins
    app.add_plugins((
        TilemapPlugin,
        ShapePlugin,
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
        crate::ui::UiPlugin,
        crate::settings::SettingsPlugin,
        crate::input::InputPlugin,
        crate::camera::MwCameraPlugin,
        crate::gfx2d::Gfx2dPlugin,
        crate::game::GameplayPlugin,
        crate::minimap::MinimapPlugin,
    ));

    app.add_plugins(mw_proprietary_client::MwProprietaryPlugin);

    #[cfg(feature = "dev")]
    app.add_plugins(crate::dev::DevPlugin);

    app.run();
}
