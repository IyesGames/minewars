use crate::camera::GridCursor;
use crate::input::GameInputSet;
use crate::input::MouseClick;
use crate::prelude::*;
use mw_app::settings::MapGenStyle;
use mw_app::view::*;
use mw_app::bevyhost::*;
use mw_app::player::*;
use mw_common::game::event::GameEvent;
use mw_game_minesweeper::*;
use mw_common::grid::*;
use mw_common::plid::*;
use mw_common::game::*;

pub struct MinesweeperGameplayPlugin;

impl Plugin for MinesweeperGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("minesweeper_singleplayer", cli_minesweeper_singleplayer);
        app.register_clicommand_noargs("minesweeper_playground", cli_minesweeper_playground);
        app.add_event::<MinesweeperInputAction>();
        app.add_plugins((
            BevyMwHostPlugin::<
                GameMinesweeper<Hex>,
                MinesweeperInputAction,
                GameEvent,
            >::new(),
            BevyMwHostPlugin::<
                GameMinesweeper<Sq>,
                MinesweeperInputAction,
                GameEvent,
            >::new(),
        ));
        app.add_systems(Update, (
            minesweeper_input
                .in_set(InGameSet(Some(GameMode::Minesweeper)))
                .in_set(GameInputSet),
        ));
    }
}
fn cli_minesweeper_singleplayer(world: &mut World) {
    world.resource_mut::<AllSettings>().game_minesweeper.n_plids = 1;
    cli_minesweeper_playground(world);
}

fn cli_minesweeper_playground(world: &mut World) {
    let minesweeper_settings = world.resource::<AllSettings>().game_minesweeper.clone();
    let mapgen_settings = world.resource::<AllSettings>().mapgen.clone();
    match (PROPRIETARY, mapgen_settings.style) {
        (false, _) | (_, MapGenStyle::Flat) => {
            match mapgen_settings.topology {
                Topology::Hex => {
                    setup_minesweeper_playground_flatmap::<Hex>(world, minesweeper_settings, mapgen_settings.size);
                }
                Topology::Sq => {
                    setup_minesweeper_playground_flatmap::<Sq>(world, minesweeper_settings, mapgen_settings.size);
                }
            }
        }
        (true, MapGenStyle::MineWars) => {
            #[cfg(feature = "proprietary")]
            mw_proprietary_client::setup_minesweeper_playground_mwmap(
                world, minesweeper_settings, mapgen_settings.size, mapgen_settings.seed,
            );
        }
    }
}

fn setup_minesweeper_playground_flatmap<C: Coord>(
    world: &mut World,
    minesweeper_settings: MinesweeperSettings,
    map_size: u8
) {
    let n_plids = minesweeper_settings.n_plids;
    let dummy_map = MapData::<C, ()>::new(map_size, ());
    mw_app::map::setup_map(world, &dummy_map, &[], |_| TileKind::Regular, |_| 0);
    let game = GameMinesweeper::<C>::new(minesweeper_settings, &dummy_map, |_| TileKind::Regular);
    world.insert_resource(BevyHost::new(game, ()));

    let mut viewtile: ViewTileData = ViewTileData::default();
    viewtile.set_owner(0);
    viewtile.set_digit(0);
    viewtile.set_kind(TileKind::Regular);
    viewtile.set_item(ItemKind::Safe);
    viewtile.set_has_structure(false);

    let e_plid0 = world.spawn(SpectatorPlidBundle::default()).id();
    let mut players_index = PlayersIndex(vec![e_plid0]);
        for i in 0..n_plids {
        let plid = PlayerId::from(i+1);
        let e_plid = world.spawn((
            PlayerBundle {
                plid: PlayerPlid(plid),
                state: PlayerState::Alive,
            },
            ViewBundle {
                mapdata: ViewMapData(MapData::<C, _>::new(map_size, viewtile)),
            },
            PlidPlayable,
        )).id();
        players_index.0.push(e_plid);
    }
    world.insert_resource(players_index);
    world.insert_resource(PlidPlayingAs(1.into()));
    world.insert_resource(PlidViewing(1.into()));

    world.resource_mut::<NextState<GameMode>>().set(GameMode::Minesweeper);
    world.resource_mut::<NextState<AppState>>().set(AppState::InGame);
    world.resource_mut::<NextState<SessionKind>>().set(SessionKind::BevyHost);
}

// TODO: replace this with something more elaborate?
fn minesweeper_input(
    crs: Res<GridCursor>,
    mut evr_mouse: EventReader<MouseClick>,
    mut evw: EventWriter<MinesweeperInputAction>,
) {
    for ev in evr_mouse.iter() {
        if ev.0 == MouseButton::Left {
            evw.send(MinesweeperInputAction::ExploreTile {
                pos: crs.0,
            });
        }
        if ev.0 == MouseButton::Right {
            evw.send(MinesweeperInputAction::ToggleFlag {
                pos: crs.0,
            });
        }
    }
}
