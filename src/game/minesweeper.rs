use crate::camera::GridCursor;
use crate::input::GameInputSet;
use crate::prelude::*;
use crate::settings::MapGenStyle;
use mw_app::map::MapTileIndex;
use mw_app::map::MapTileIndexCoord;
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
    let mut minesweeper_settings = world.resource::<AllSettings>().game_minesweeper.clone();
    minesweeper_settings.n_plids = 1;
    let mapgen_settings = world.resource::<AllSettings>().mapgen.clone();
    match (PROPRIETARY, mapgen_settings.style) {
        (false, _) | (_, MapGenStyle::Flat) => {
            match mapgen_settings.topology {
                Topology::Hex => {
                    setup_minesweeper_singleplayer_flatmap::<Hex>(world, minesweeper_settings, mapgen_settings.size);
                }
                Topology::Sq => {
                    setup_minesweeper_singleplayer_flatmap::<Sq>(world, minesweeper_settings, mapgen_settings.size);
                }
            }
        }
        (true, MapGenStyle::MineWars) => {
            #[cfg(feature = "proprietary")]
            mw_proprietary_client::setup_minesweeper_singleplayer_mwmap(
                world, minesweeper_settings, mapgen_settings.size, mapgen_settings.seed,
            );
        }
    }
}

fn setup_minesweeper_singleplayer_flatmap<C: MapTileIndexCoord>(
    world: &mut World,
    mut minesweeper_settings: MinesweeperSettings,
    map_size: u8
) {
    minesweeper_settings.n_plids = 1;

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
    let e_plid1 = world.spawn((
        PlayerBundle {
            plid: PlayerPlid(1.into()),
            state: PlayerState::Alive,
        },
        ViewBundle {
            mapdata: ViewMapData(MapData::<C, _>::new(map_size, viewtile)),
        },
    )).id();
    world.insert_resource(PlayersIndex(vec![e_plid0, e_plid1]));
    world.insert_resource(PlidPlayingAs(1.into()));
    world.insert_resource(PlidViewing(1.into()));

    world.resource_mut::<NextState<GameMode>>().set(GameMode::Minesweeper);
    world.resource_mut::<NextState<AppState>>().set(AppState::InGame);
    world.resource_mut::<NextState<SessionKind>>().set(SessionKind::BevyHost);
}

// TODO: replace this with something more elaborate?
fn minesweeper_input(
    crs: Res<GridCursor>,
    btn: Res<Input<MouseButton>>,
    mut evw: EventWriter<MinesweeperInputAction>,
) {
    if btn.just_pressed(MouseButton::Left) {
        evw.send(MinesweeperInputAction::ExploreTile {
            pos: crs.0,
        });
    }
    if btn.just_pressed(MouseButton::Middle) {
        evw.send(MinesweeperInputAction::SetFlag {
            flag: true,
            pos: crs.0,
        });
    }
}

