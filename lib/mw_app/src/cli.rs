use mw_app_core::{driver::*, player::*, session::*, user::*};

use crate::{map::SimpleMapGenerator, prelude::*, settings::{MapGenSettings, PlidColorSettings}};

pub fn plugin(app: &mut App) {
    app.register_clicommand_noargs(
        "start_minesweeper_singleplayer",
        start_minesweeper_singleplayer
    );
}

fn start_minesweeper_singleplayer(
    // In(args): In<Vec<String>>, 
    mut commands: Commands,
    settings: Settings,
    mut state: ResMut<NextState<AppState>>,
    q_user: Query<&MyUserProfile, With<UserGovernor>>,
) {
    let s_mapgen = settings.get::<MapGenSettings>().unwrap();
    let s_colors = settings.get::<PlidColorSettings>().unwrap();

    let e_subplid = commands.spawn((
        SubPlidBundle::new(0, &q_user.single().0),
    )).id();
    let e_plid0 = commands.spawn((
        SpectatorPlidBundle::default(),
    )).id();
    let e_plid1 = commands.spawn((
        PlayerPlidBundle::new(1.into(), s_colors.colors[1].into(), &[e_subplid]),
    )).id();
    commands.spawn((
        SessionGovernorBundle::new(
            1.into(), &[e_plid0, e_plid1], &[&[], &[e_subplid]]
        ),
    ));
    commands.spawn((
        DriverGovernorBundle::default(),
        SimpleMapGenerator {
            topology: s_mapgen.topology,
            size: s_mapgen.size,
        },
        // TODO: Bevy Driver
    ));

    state.set(AppState::GameLoading);
}

