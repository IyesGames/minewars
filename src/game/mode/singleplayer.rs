use mw_common::game::MapDataInitAny;
use mw_common::grid::*;
use mw_game_classic::*;

use crate::camera::GridCursor;
use crate::prelude::*;
use crate::game::skip_lobby_state;

pub struct GameModeSingleplayerPlugin;

impl Plugin for GameModeSingleplayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Singleplayer)
        );
        app.add_exit_system(
            AppGlobalState::GameLoading,
            init_game_impl.run_in_state(GameMode::Singleplayer)
        );
        app.add_system(
            mouse_input
                .run_in_state(AppGlobalState::InGame)
                .run_in_state(GameMode::Singleplayer)
                .after("cursor")
                .before(MwLabels::HostInEvents)
        );
    }
}

fn init_game_impl(
    mut commands: Commands,
    data: Res<MapDataInitAny>,
) {
    match data.map.topology () {
        Topology::Hex => {
            debug!("poiiong");
            commands.insert_resource(
                MwClassicSingleplayerGame::<Hex>::new_with_map(&*data, None).unwrap()
            );
        },
        Topology::Sq => {
            commands.insert_resource(
                MwClassicSingleplayerGame::<Sq>::new_with_map(&*data, None).unwrap()
            );
        },
        Topology::Sqr => { unimplemented!() },
    }
}

fn mouse_input(
    mut evw_game: EventWriter<InputAction>,
    crs: Res<GridCursor>,
    input: Res<Input<MouseButton>>,
) {
    if input.just_pressed(MouseButton::Left) {
        evw_game.send(InputAction::ExploreTileSingle { tile: crs.0 });
    }
}
