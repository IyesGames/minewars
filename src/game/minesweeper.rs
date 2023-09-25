use crate::prelude::*;
use crate::settings::MapGenStyle;
use crate::view::PlidViewing;
use crate::view::ViewBundle;
use crate::view::ViewMapData;
use crate::view::ViewTileData;
use mw_app::bevyhost::*;
use mw_app::player::*;
use mw_game_minesweeper::*;
use mw_common::grid::*;
use mw_common::plid::*;
use mw_common::game::*;

pub struct MinesweeperGameplayPlugin;

impl Plugin for MinesweeperGameplayPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("minesweeper_singleplayer", cli_minesweeper_singleplayer);
        app.add_event::<MinesweeperInputAction>();
        app.add_event::<MinesweeperOutEvent>();
        app.add_plugins((
            BevyMwHostPlugin::<
                GameMinesweeper<Hex>,
                MinesweeperInputAction,
                MinesweeperOutEvent,
            >::new(),
            BevyMwHostPlugin::<
                GameMinesweeper<Sq>,
                MinesweeperInputAction,
                MinesweeperOutEvent,
            >::new(),
        ));
    }
}

#[derive(Event, Debug, Clone)]
pub struct MinesweeperOutEvent {
    plid: PlayerId,
    ev: MinesweeperEvent,
}

impl From<(PlayerId, MinesweeperEvent)> for MinesweeperOutEvent {
    fn from(value: (PlayerId, MinesweeperEvent)) -> Self {
        Self {
            plid: value.0,
            ev: value.1,
        }
    }
}

fn cli_minesweeper_singleplayer(world: &mut World) {
    let mut minesweeper_settings = world.resource::<AllSettings>().game_minesweeper.clone();
    minesweeper_settings.n_plids = 1;
    let mapgen_settings = world.resource::<AllSettings>().mapgen.clone();
    match mapgen_settings.style {
        // TODO
        _ => {
            match mapgen_settings.topology {
                Topology::Hex => {
                    setup_minesweeper_singleplayer_flatmap::<Hex>(world);
                }
                Topology::Sq => {
                    setup_minesweeper_singleplayer_flatmap::<Sq>(world);
                }
            }
        }
        // MapGenStyle::MineWars => {
        //     todo!();
        // }
    }
}

fn setup_minesweeper_singleplayer_flatmap<C: Coord>(world: &mut World) {
    let mut minesweeper_settings = world.resource::<AllSettings>().game_minesweeper.clone();
    minesweeper_settings.n_plids = 1;
    let mapgen_settings = world.resource::<AllSettings>().mapgen.clone();

    let dummy_map = MapData::<C, ()>::new(mapgen_settings.size, ());
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
            mapdata: ViewMapData(MapData::<C, _>::new(mapgen_settings.size, viewtile)),
        },
    )).id();
    world.insert_resource(PlayersIndex(vec![e_plid0, e_plid1]));
    world.insert_resource(PlidPlayingAs(1.into()));
    world.insert_resource(PlidViewing(1.into()));

    world.resource_mut::<NextState<AppState>>().set(AppState::InGame);
    world.resource_mut::<NextState<SessionKind>>().set(SessionKind::BevyHost);
}

