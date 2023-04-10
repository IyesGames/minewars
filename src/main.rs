mod prelude {
    pub use iyesengine::prelude::*;
    pub use anyhow::{Context, Result as AnyResult, Error as AnyError, anyhow, bail, ensure};
    pub use rand::prelude::*;
    pub use mw_common::{HashMap, HashSet};
    pub use mw_common::app::*;
    pub use std::time::{Duration, Instant};
    pub use crate::PROPRIETARY;
}

use crate::prelude::*;

use bevy::window::{PresentMode, WindowResizeConstraints};
use bevy::log::LogSettings;

mod assets;
mod camera;
mod game;
mod map;
mod ui;
mod settings;

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

fn main() {
    #[cfg(feature = "proprietary")]
    if let Err(e) = mw_proprietary::init() {
        error!("Init error: {:#}", e);
        std::process::exit(42);
    }

    let mut app = App::new();

    #[cfg(feature = "dev")]
    app.insert_resource(LogSettings {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=trace".into(),
        level: bevy::log::Level::TRACE,
    });
    #[cfg(not(feature = "dev"))]
    app.insert_resource(LogSettings {
        filter: "info,wgpu_core=warn,wgpu_hal=warn,minewars=info".into(),
        level: bevy::log::Level::INFO,
    });
    app.insert_resource(WindowDescriptor {
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
        // scale_factor_override: Some(1.0),
        ..Default::default()
    });
    app.insert_resource(ClearColor(Color::BLACK));
    app.add_plugin(IyesEverything);
    // FIXME: these should be handled as settings
    app.insert_resource(mw_common::game::MapDescriptor {
        size: 32,
        topology: mw_common::grid::Topology::Hex,
    });
    app.insert_resource(crate::map::MwMapGfxBackend::Tilemap);

    app.add_loopless_state(StreamSource::Disconnected);
    app.add_loopless_state(GameMode::None);
    app.add_loopless_state(AppGlobalState::AssetsLoading);
    app.add_event::<mw_common::app::GamePlayerEvent>();
    app.add_plugin(crate::settings::SettingsPlugin);
    app.add_plugin(crate::ui::UiPlugin);
    app.add_plugin(crate::assets::AssetsPlugin);
    app.add_plugin(crate::ui::mainmenu::MainMenuPlugin);
    app.add_plugin(crate::camera::CameraPlugin);
    app.add_plugin(crate::map::MapPlugin);
    app.add_plugin(crate::game::GamePlugin);

    app.add_event::<mw_game_classic::InputAction>();
    app.add_plugin(mw_common::host::BevyMwHostPlugin::<
        mw_game_classic::MwClassicSingleplayerGame::<mw_common::grid::Hex>,
        mw_game_classic::InputAction,
        mw_common::app::GamePlayerEvent,
        GameMode,
    >::new(GameMode::Singleplayer));

    app.add_system(debug_current_state);

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
        debug!("State: {:?} / {:?} / {:?}!", app.0, mode.0, src.0);
    }
}
