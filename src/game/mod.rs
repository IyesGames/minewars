use crate::prelude::*;

mod minesweeper;
mod minewars;

pub struct GameplayPlugin;

impl Plugin for GameplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            minesweeper::MinesweeperGameplayPlugin,
            minewars::MinewarsGameplayPlugin,
        ));
    }
}
