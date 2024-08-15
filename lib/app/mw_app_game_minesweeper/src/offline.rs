use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use mw_app_core::{driver::{DriverGovernor, NeedsDriverGovernorSet}, map::{MapDataOrig, MapDescriptor, MapGovernor, MapTileDataOrig}, session::{NeedsSessionGovernorSet, PlayersIndex, SessionGovernor}};
use mw_app_io::offline_host::OfflineHost;
use mw_game_minesweeper::{builder::GameMinesweeperBuilder, minegen::MineGenSettings, GameMinesweeper, MinesweeperInitData, MinesweeperSettings};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update,
        setup_offline_game
            .track_progress()
            .in_set(InStateSet(AppState::GameLoading))
            .in_set(NeedsSessionGovernorSet)
            .in_set(NeedsDriverGovernorSet)
            .run_if(any_filter::<(With<SetupOfflineGame>, With<DriverGovernor>)>)
    );
}

#[derive(Component)]
pub struct SetupOfflineGame {
    pub settings: MinesweeperSettings,
    pub minegen: MineGenSettings,
}

#[derive(Default)]
enum SetupState {
    #[default]
    NotStarted,
    Awaiting(Task<Box<GameMinesweeper>>),
    Done,
}

fn setup_offline_game(
    mut commands: Commands,
    q_session: Query<&PlayersIndex, With<SessionGovernor>>,
    q_driver: Query<(Entity, &SetupOfflineGame), With<DriverGovernor>>,
    q_map: Query<(&MapDescriptor, &MapDataOrig), With<MapGovernor>>,
    mut state: Local<SetupState>,
) -> Progress {
    match &mut *state {
        SetupState::NotStarted => {
            let Ok((mapdesc, mapdata)) = q_map.get_single() else {
                return false.into();
            };
            let n_plids = q_session.single().e_plid.len() - 1;
            let (_, setup) = q_driver.single();
            let rt = AsyncComputeTaskPool::get();
            match mapdesc.topology {
                Topology::Hex => {
                    let settings = setup.settings.clone();
                    let mapdata = mapdata.map.clone();
                    let task = rt.spawn(async move {
                        GameMinesweeperBuilder::new(settings, n_plids as u8)
                            .with_mapdata_hex(mapdata.size(), |c| mapdata[c.into()].kind())
                    });
                    *state = SetupState::Awaiting(task);
                }
                Topology::Sq => {
                    let settings = setup.settings.clone();
                    let mapdata = mapdata.map.clone();
                    let task = rt.spawn(async move {
                        GameMinesweeperBuilder::new(settings, n_plids as u8)
                            .with_mapdata_sq(mapdata.size(), |c| mapdata[c.into()].kind())
                    });
                    *state = SetupState::Awaiting(task);
                }
            }
            false.into()
        }
        SetupState::Awaiting(task) => {
            if let Some(game) = block_on(poll_once(task)) {
                let (e_driver, setup) = q_driver.single();
                commands.entity(e_driver).insert((
                    OfflineHost::new(game, Box::new(MinesweeperInitData {
                        minegen: setup.minegen.clone()
                    })),
                ));
                *state = SetupState::Done;
                true.into()
            } else {
                false.into()
            }
        }
        SetupState::Done => true.into(),
    }
}
