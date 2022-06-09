use crate::prelude::*;
use crate::{AppGlobalState, GameMode};
use crate::game::skip_lobby_state;

pub struct GameModeDevPlugin;

impl Plugin for GameModeDevPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Dev)
        );
    }
}
