use bevy::app::AppExit;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_clicommand_noargs("softreset", softreset);
    app.register_clicommand_noargs("exit_game", exit_game);
    app.register_clicommand_noargs("exit_app", exit_app);
}

fn softreset(
    mut s: ResMut<NextState<AppState>>,
) {
    s.set(AppState::GameLoading);
}

fn exit_game(
    mut s: ResMut<NextState<AppState>>,
) {
    s.set(AppState::Menu);
}

fn exit_app(
    mut evw: EventWriter<AppExit>,
) {
    evw.send(AppExit::Success);
}
