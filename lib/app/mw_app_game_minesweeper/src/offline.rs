use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use mw_app_core::{driver::{DriverGovernor, NeedsDriverGovernorSet}, map::{MapDataOrig, MapDescriptor, MapGovernor, MapTileDataOrig}, session::NeedsSessionGovernorSet};
use mw_app_io::offline_host::OfflineHost;
use mw_game_minesweeper::{GameMinesweeper, MinesweeperSettings};

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
}

#[derive(Default)]
enum SetupState {
    #[default]
    NotStarted,
    AwaitingHex(Task<Box<GameMinesweeper<Hex>>>),
    AwaitingSq(Task<Box<GameMinesweeper<Sq>>>),
    Done,
}

fn setup_offline_game(
    mut commands: Commands,
    q_driver: Query<(Entity, &SetupOfflineGame), With<DriverGovernor>>,
    q_map: Query<(&MapDescriptor, &MapDataOrig), With<MapGovernor>>,
    mut state: Local<SetupState>,
) -> Progress {
    match &mut *state {
        SetupState::NotStarted => {
            let Ok((mapdesc, mapdata)) = q_map.get_single() else {
                return false.into();
            };
            let (_, setup) = q_driver.single();
            let rt = AsyncComputeTaskPool::get();
            match mapdesc.topology {
                Topology::Hex => {
                    let settings = setup.settings.clone();
                    let mapdata = mapdata.map.clone();
                    let task = rt.spawn(async move {
                        let game = GameMinesweeper::<Hex>::new(
                            settings, &mapdata.rekey(),
                            |c: &MapTileDataOrig| c.kind()
                        );
                        Box::new(game)
                    });
                    *state = SetupState::AwaitingHex(task);
                }
                Topology::Sq => {
                    let settings = setup.settings.clone();
                    let mapdata = mapdata.map.clone();
                    let task = rt.spawn(async move {
                        let game = GameMinesweeper::<Sq>::new(
                            settings, &mapdata.rekey(),
                            |c: &MapTileDataOrig| c.kind()
                        );
                        Box::new(game)
                    });
                    *state = SetupState::AwaitingSq(task);
                }
            }
            false.into()
        }
        SetupState::AwaitingHex(task) => {
            if let Some(game) = block_on(poll_once(task)) {
                let (e_driver, _) = q_driver.single();
                commands.entity(e_driver).insert((
                    OfflineHost::new(game, None),
                ));
                *state = SetupState::Done;
                true.into()
            } else {
                false.into()
            }
        }
        SetupState::AwaitingSq(task) => {
            if let Some(game) = block_on(poll_once(task)) {
                let (e_driver, _) = q_driver.single();
                commands.entity(e_driver).insert((
                    OfflineHost::new(game, None),
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
