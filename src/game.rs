use mw_common::game::*;
use mw_common::grid::*;
use mw_common::plid::*;

use crate::prelude::*;

use crate::GameMode;
use crate::StreamSource;
use crate::AppGlobalState;

mod mode {
    #[cfg(feature = "dev")]
    pub mod dev;
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(
            ProgressPlugin::new(AppGlobalState::GameLoading)
                .continue_to(AppGlobalState::InGame)
        );
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
            skip_lobby_state.run_in_state(GameMode::Singleplayer)
        );
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Spectator)
        );
        app.add_enter_system(
            AppGlobalState::GameLobby,
            skip_lobby_state.run_in_state(GameMode::Tutorial)
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

fn world_gen_system(
    mut commands: Commands,
    mapdesc: Res<MapDescriptor>,
    mut done: Local<bool>,
) -> Progress {
    if !*done {
        // TODO: have this configurable somewhere
        let n_mines = mapdesc.size as u32 * mapdesc.size as u32 / 3;
        match mapdesc.topology {
            Topology::Hex => commands.insert_resource(world_gen_flat::<Hex>(mapdesc.size, n_mines)),
            Topology::Sq => commands.insert_resource(world_gen_flat::<Sq>(mapdesc.size, n_mines)),
            Topology::Sqr => commands.insert_resource(world_gen_flat::<Sqr>(mapdesc.size, n_mines)),
        };
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
) -> MapDataInit<C> {
    assert!(n_mines < size as u32 * size as u32);

    let mut rng = thread_rng();

    let mut data = MapDataInit {
        map: MapData::new(size, MapTileInit {
            kind: TileKind::Regular,
            mine: None,
            region: 0xff,
            cit: false,
            mark: false,
        }),
        cits: Default::default(),
        mines: Default::default(),
    };

    let size = size as i8;

    // populate with random mines
    while n_mines > 0 {
        let y = rng.gen_range(-size..=size);
        let x = rng.gen_range(C::xmin(size as u8, y)..=C::xmax(size as u8, y));

        let pos = Pos(x, y).into();

        if data.map[pos].mine.is_none() {
            data.map[pos].mine = Some(MineKind::Mine);
            data.mines.insert(pos, MineKind::Mine);
            n_mines -= 1;
        }
    }

    data
}
