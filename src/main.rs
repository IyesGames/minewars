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
mod ui;

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
enum AppGlobalState {
    AssetsLoading,
    SplashIyes,
    SplashBevy,
    MainMenu,
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
        .add_loopless_state(AppGlobalState::AssetsLoading)
        .add_plugin(crate::ui::UiPlugin)
        .add_plugin(crate::assets::AssetsPlugin)
        .add_plugin(crate::ui::mainmenu::MainMenuPlugin)
        .add_system(debug_current_state)
        ;

    #[cfg(feature = "proprietary")]
    app.add_plugin(mw_proprietary::ClientPlugin);

    app.run();
}

fn debug_current_state(state: Res<CurrentState<AppGlobalState>>) {
    if state.is_changed() {
        debug!("Detected state change to {:?}!", state);
    }
}
