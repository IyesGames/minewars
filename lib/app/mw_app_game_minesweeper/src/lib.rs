pub mod prelude {
    pub use mw_app_core::prelude::*;
}

mod cli;
mod input;
mod map;
pub mod offline;
pub mod settings;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        mw_app_io::offline_host::OfflineHostPlugin::<
            Box<mw_game_minesweeper::GameMinesweeper>,
            mw_game_minesweeper::MinesweeperInputAction,
            mw_common::game::GameEvent,
        >::new(),
    ));
    app.add_plugins((
        crate::cli::plugin,
        crate::input::plugin,
        crate::map::plugin,
        crate::offline::plugin,
        crate::settings::plugin,
    ));
    app.add_event::<mw_game_minesweeper::MinesweeperInputAction>();
}
