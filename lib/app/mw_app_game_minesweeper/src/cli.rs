use mw_app_core::{driver::*, graphics::*, player::*, session::*, settings::{GraphicsStyleSettings, PlidColorSettings}, user::*};
use mw_game_minesweeper::MinesweeperSettings;

use crate::{map::SimpleMapGenerator, offline::SetupOfflineGame, prelude::*, settings::SimpleMapSettings};

pub fn plugin(app: &mut App) {
    app.register_clicommand_noargs(
        "start_minesweeper_singleplayer",
        start_minesweeper_singleplayer
    );
}

fn start_minesweeper_singleplayer(
    mut commands: Commands,
    settings: Settings,
    mut state: ResMut<NextState<AppState>>,
    q_user: Query<&MyUserProfile, With<UserGovernor>>,
) {
    let s_mapgen = settings.get::<SimpleMapSettings>().unwrap();
    let s_colors = settings.get::<PlidColorSettings>().unwrap();
    let s_gfx = settings.get::<GraphicsStyleSettings>().unwrap();

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
        SetupOfflineGame {
            settings: MinesweeperSettings {
                n_plids: 1,
                ..Default::default()
            },
        },
    ));
    let e_gov_gfx = commands.spawn((
        GraphicsGovernorBundle {
            cleanup: default(),
            marker: GraphicsGovernor,
            style: CurrentGraphicsStyle(s_gfx.game_preferred_style),
        },
        DisplayDigitsMode::Game,
        DisplayItemsMode::MyItems,
    )).id();
    if s_gfx.game_enable_both_styles {
        commands.entity(e_gov_gfx).insert((
            Gfx2dEnabled,
            Gfx3dEnabled,
        ));
    } else {
        match s_gfx.game_preferred_style {
            GraphicsStyle::Gfx2d => commands.entity(e_gov_gfx)
                .insert(Gfx2dEnabled),
            GraphicsStyle::Gfx3d => commands.entity(e_gov_gfx)
                .insert(Gfx3dEnabled),
        };
    }

    state.set(AppState::GameLoading);
}
