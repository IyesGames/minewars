mod prelude {
    pub use mw_common::prelude::*;
    pub use mw_common::grid::*;
    pub use mw_common::plid::*;
    pub use mw_common::algo::*;
}

#[cfg(feature = "bevy")]
pub mod bevy_stuff;

use modular_bitfield::prelude::*;
use rand::prelude::*;
use mw_common::{game::{ItemKind, TileKind}, driver::{Game, Host}};

use crate::prelude::*;

/// Settings that can be configured for a session of the Minesweeper game mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinesweeperSettings {
    /// Each plid will get eliminated from the game when they step on a mine this many times.
    pub n_lives: u8,
    /// 1 for singleplayer or co-op. >1 for PvP, Duos, etc.
    pub n_plids: u8,
    /// If nonzero, limit the maximum time allowed for the game.
    pub time_limit_secs: u16,
    /// Probability of a mine appearing on a tile.
    pub mine_density: u8,
    /// Probability a mine being replaced by a decoy instead.
    pub prob_decoy: u8,
}

impl Default for MinesweeperSettings {
    fn default() -> Self {
        Self {
            n_lives: 1,
            n_plids: 1,
            time_limit_secs: 0,
            mine_density: 80,
            prob_decoy: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub enum MinesweeperInputAction {
    ExploreTile {
        pos: Pos,
    },
    SetFlag {
        pos: Pos,
        flag: bool,
    },
}

#[derive(Debug, Clone)]
pub enum PlayerEvent {
    Exploded {
        pos: Pos,
        lives_remain: u8,
    },
    Eliminated,
}

#[derive(Debug, Clone)]
pub enum MinesweeperEvent {
    Player(PlayerEvent),
    TileOwner {
        pos: Pos,
        plid: PlayerId,
    },
    SetFlag {
        pos: Pos,
        /// Neutral means remove flag. Other values mean that the plid placed a flag.
        plid: PlayerId,
    },
}

pub struct GameMinesweeper<C: Coord> {
    settings: MinesweeperSettings,
    mapdata: MapData<C, TileData>,
}

impl<C: Coord> GameMinesweeper<C> {
    pub fn new<D>(settings: MinesweeperSettings, map_src: &MapData<C, D>, f_tilekind: impl Fn(&D) -> TileKind) -> Self {
        let mut rng = thread_rng();
        let mapdata = map_src.convert(|_, d| {
            let mut tile = TileData::default();
            tile.set_owner(0);
            tile.set_flag(0);
            let tilekind = f_tilekind(&d);
            tile.set_land(tilekind.is_land());
            tile.set_cluster(tilekind.is_rescluster());
            if tilekind.is_land() {
                let item = if rng.gen_bool(settings.mine_density as f64 / 255.0) {
                    if rng.gen_bool(settings.prob_decoy as f64 / 255.0) {
                        ItemKind::Decoy
                    } else {
                        ItemKind::Mine
                    }
                } else {
                    ItemKind::Safe
                };
                tile.set_item(item);
            } else {
                tile.set_item(ItemKind::Safe);
            }
            tile
        });
        Self {
            settings,
            mapdata,
        }
    }
}

#[bitfield]
#[derive(Clone, Copy, Default)]
struct TileData {
    owner: B4,
    land: bool,
    cluster: bool,
    item: ItemKind,
    flag: B4,
    #[skip] __: B4,
}

impl<C: Coord> Game for GameMinesweeper<C> {
    type InitData = ();
    type InputAction = MinesweeperInputAction;
    type OutEvent = MinesweeperEvent;
    type SchedEvent = ();

    fn init<H: Host<Self>>(&mut self, host: &mut H, initdata: Self::InitData) {
    }
    fn input<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, action: Self::InputAction) {
    }
    fn unsched<H: Host<Self>>(&mut self, host: &mut H, event: Self::SchedEvent) {
    }
}
