use mw_common::game::*;
use mw_common::grid::*;
use mw_common::plid::*;
use mw_common::proto::GameEvent;

use crate::map::MapEvent;
use crate::map::MapEventKind;
use crate::map::MapLabels;
use crate::map::MineDisplayState;
use crate::prelude::*;

use crate::GameMode;
use crate::StreamSource;
use crate::AppGlobalState;

mod mode {
    #[cfg(feature = "dev")]
    pub mod dev;
    pub mod singleplayer;
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(
            ProgressPlugin::new(AppGlobalState::GameLoading)
                .continue_to(AppGlobalState::InGame)
        );
        app.add_exit_system(AppGlobalState::InGame, remove_resource::<GameParams>);
        // For "local" games, we need to generate a world map
        // (fallback for open-source builds,
        // proprietary takes care of this by itself)
        #[cfg(not(feature = "proprietary"))]
        app.add_system(
            world_gen_system
                .track_progress()
                .run_in_state(AppGlobalState::GameLoading)
                .run_in_state(StreamSource::Local)
        );
        app.add_enter_system(
            AppGlobalState::GameLoading,
            init_local_as_player1
                .run_in_state(StreamSource::Local)
        );
        // drop any generated init map data when starting game
        // everything needing it should be done in the GameLoading state
        app.add_enter_system(
            AppGlobalState::InGame,
            remove_resource::<MapDataInitAny>
        );
        app.add_plugin(mode::singleplayer::GameModeSingleplayerPlugin);
        #[cfg(feature = "dev")]
        app.add_plugin(mode::dev::GameModeDevPlugin);
        // FIXME: temporary; replace with actual lobby implementations
        // for different game modes
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Multiplayer)
        );
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Spectator)
        );
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Tutorial)
        );
        app.add_system(game_event_to_map_event
            .after(MwLabels::HostOutEvents)
            .before(MapLabels::ApplyEvents)
        );
    }
}

fn skip_lobby_state(mut commands: Commands) {
    commands.insert_resource(NextState(AppGlobalState::GameLoading));
}

fn init_local_as_player1(
    mut commands: Commands,
) {
    commands.insert_resource(ActivePlid(PlayerId::from(1)));
}

fn game_event_to_map_event(
    mut evr: EventReader<GamePlayerEvent>,
    mut evw: EventWriter<MapEvent>,
) {
    for ev in evr.iter() {
        match ev.event {
            GameEvent::Owner { tile, owner } => {
                evw.send(MapEvent {
                    plid: ev.plid, c: tile,
                    kind: MapEventKind::Owner {
                        plid: owner,
                    },
                });
            }
            GameEvent::Digit { tile, digit } => {
                evw.send(MapEvent {
                    plid: ev.plid, c: tile,
                    kind: MapEventKind::Digit {
                        digit,
                    },
                });
            }
            GameEvent::Mine { tile, kind } => {
                evw.send(MapEvent {
                    plid: ev.plid, c: tile,
                    kind: MapEventKind::Mine {
                        state: kind.map(|k| MineDisplayState::Normal(k)),
                    },
                });
            }
            GameEvent::MineActive { tile } => {
                evw.send(MapEvent {
                    plid: ev.plid, c: tile,
                    kind: MapEventKind::Mine {
                        state: Some(MineDisplayState::Active),
                    },
                });
            }
            GameEvent::Explosion { tile, kind } => {
                evw.send(MapEvent {
                    plid: ev.plid, c: tile,
                    kind: MapEventKind::Explosion {
                        kind,
                    },
                });
            }
            GameEvent::Road { tile, state } => {
                unimplemented!()
            }
            _ => {}
        }
    }
}

fn world_gen_system(
    mut commands: Commands,
    mapdesc: Res<MapDescriptor>,
    mut done: Local<bool>,
) -> Progress {
    if !*done {
        // TODO: have this configurable somewhere
        let n_mines = mapdesc.size as u32 * mapdesc.size as u32 / 3;
        let mapinit = match mapdesc.topology {
            Topology::Hex => world_gen_flat::<Hex>(mapdesc.size, n_mines),
            Topology::Sq => world_gen_flat::<Sq>(mapdesc.size, n_mines),
            Topology::Sqr => world_gen_flat::<Sqr>(mapdesc.size, n_mines),
        };
        commands.insert_resource(mapinit);
        *done = true;
    }

    (*done).into()
}

/// "Flat" world generation (no geography, plain grid)
///
/// Provided as an option for singleplayer mode.
/// Used as fallback for open-source builds without proprietary worldgen.
fn world_gen_flat<C: CompactMapCoordExt>(
    size: u8,
    mut n_mines: u32,
) -> MapDataInitAny {
    assert!(n_mines < size as u32 * size as u32);

    let mut rng = thread_rng();

    let mut map = MapData::new(size, MapTileInit {
        kind: TileKind::Regular,
        mine: None,
        region: 0xff,
        cit: false,
        mark: false,
    });
    let cits: Vec<Pos> = Default::default();
    let mut mines: HashMap<Pos, MineKind> = Default::default();

    let size = size as i8;

    // populate with random mines
    while n_mines > 0 {
        let y = rng.gen_range(-size..=size);
        let x = rng.gen_range(C::xmin(size as u8, y)..=C::xmax(size as u8, y));

        let pos = Pos(x, y);
        let c: C = pos.into();

        if map[c].mine.is_none() {
            map[c].mine = Some(MineKind::Mine);
            mines.insert(pos, MineKind::Mine);
            n_mines -= 1;
        }
    }

    MapDataInitAny {
        map: MapAny::from(map),
        cits,
        mines,
    }
}
