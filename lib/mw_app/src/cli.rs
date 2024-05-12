use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_clicommand_args(
        "start_minesweeper_singleplayer",
        start_minesweeper_singleplayer
    );
}

fn start_minesweeper_singleplayer(
    In(args): In<Vec<String>>, 
    mut commands: Commands,
    settings: Settings,
) {
}
