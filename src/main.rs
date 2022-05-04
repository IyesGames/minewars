mod prelude {
    pub use iyesengine::prelude::*;
    pub use anyhow::{Context, Result as AnyResult, Error as AnyError};
    pub use bevy::utils::{HashMap, HashSet};
    pub use crate::PROPRIETARY;
}

use crate::prelude::*;

use bevy::window::{PresentMode, WindowResizeConstraints};
use bevy::log::LogSettings;

mod assets;
mod camera;
mod map;
mod ui;

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

/// State type: If we are in-game, where is the gameplay data coming from?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum StreamSource {
    /// No active data source
    Disconnected,
    /// Network server
    Connected,
    /// We are the server (LAN)
    Host,
    /// Hosting for ourselves only (offline)
    Local,
    /// Replay file
    File,
}

/// State type: If we are in-game, which mode are we in?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum GameMode {
    /// Not in game
    None,
    /// Playing multiplayer
    /// (see also [`StreamSource`])
    Multiplayer,
    /// Watching (replays or live)
    /// (see also [`StreamSource`])
    Spectator,
    /// Tutorial
    Tutorial,
    #[cfg(feature = "dev")]
    /// Developer Mode
    Dev,
}

/// State type: Which "screen" is the app in?
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum AppGlobalState {
    /// Initial loading screen at startup
    AssetsLoading,
    /// Splash with the IyesGames logo
    SplashIyes,
    /// Splash with the Bevy logo
    SplashBevy,
    /// Main Menu
    MainMenu,
    /// Gameplay
    /// (see also [`GameMode`])
    InGame,
}

fn main() {
    #[cfg(feature = "proprietary")]
    if let Err(e) = mw_proprietary::init() {
        error!("Init error: {:#}", e);
    }

    let mut app = App::new();

    app
        .insert_resource(LogSettings {
            filter: "warn,minewars=debug".into(),
            level: bevy::log::Level::DEBUG,
        })
        .insert_resource(WindowDescriptor {
            title: "MineWars™ PRE-ALPHA".into(),
            present_mode: PresentMode::Fifo,
            resizable: true,
            width: 800.0,
            height: 600.0,
            resize_constraints: WindowResizeConstraints {
                min_width: 800.0,
                min_height: 600.0,
                max_width: f32::INFINITY,
                max_height: f32::INFINITY,
            },
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(IyesEverything)
        // FIXME: for testing
        .insert_resource(crate::map::MapDescriptor {
            size: 5,
            topology: mw_common::grid::Topology::Hex,
        })
        .add_loopless_state(StreamSource::Disconnected)
        .add_loopless_state(GameMode::None)
        .add_loopless_state(AppGlobalState::AssetsLoading)
        .add_plugin(crate::ui::UiPlugin)
        .add_plugin(crate::assets::AssetsPlugin)
        .add_plugin(crate::ui::mainmenu::MainMenuPlugin)
        .add_plugin(crate::camera::CameraPlugin)
        .add_plugin(crate::map::MapPlugin)
        .add_system(debug_current_state)
        ;

    #[cfg(feature = "proprietary")]
    app.add_plugin(mw_proprietary::ClientPlugin);

    app.run();
}

fn debug_current_state(
    app: Res<CurrentState<AppGlobalState>>,
    mode: Res<CurrentState<GameMode>>,
    src: Res<CurrentState<StreamSource>>,
) {
    if app.is_changed() || mode.is_changed() || src.is_changed() {
        debug!("State: {:?} / {:?} / {:?}!", app, mode, src);
    }
}
