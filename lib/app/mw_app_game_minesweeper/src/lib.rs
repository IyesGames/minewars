pub mod prelude {
    pub use mw_app_core::prelude::*;
}

mod cli;
mod settings;
mod input;
mod map;
pub mod offline;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        mw_app_io::offline_host::OfflineHostPlugin::<
            mw_game_minesweeper::GameMinesweeperTopo<Hex>,
            mw_game_minesweeper::MinesweeperInputAction,
            mw_common::game::GameEvent,
        >::new(),
        mw_app_io::offline_host::OfflineHostPlugin::<
            mw_game_minesweeper::GameMinesweeperTopo<Sq>,
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
