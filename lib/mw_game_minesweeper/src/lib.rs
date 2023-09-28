use mw_common::prelude::*;
use mw_common::grid::*;
use mw_common::plid::*;
use mw_common::algo::*;
use mw_common::game::*;
use mw_common::game::event::*;
use mw_common::driver::*;

use modular_bitfield::prelude::*;

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
            mine_density: 96,
            prob_decoy: 48,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Event))]
pub enum MinesweeperInputAction {
    ExploreTile {
        pos: Pos,
    },
    SetFlag {
        pos: Pos,
        flag: bool,
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
    type OutEvent = MwEv;
    type SchedEvent = ();

    fn init<H: Host<Self>>(&mut self, host: &mut H, initdata: Self::InitData) {
    }
    fn input<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, action: Self::InputAction) {
        if u8::from(plid) > self.settings.n_plids || plid == PlayerId::Neutral {
            return;
        }
        match action {
            MinesweeperInputAction::ExploreTile { pos } => {
                self.explore_tile(host, plid, pos.into());
            }
            MinesweeperInputAction::SetFlag { pos, flag } => {
                host.msg(Plids::all(true), MwEv::Map {
                    pos,
                    ev: MapEv::Flag {
                        plid: if flag { plid } else { PlayerId::Neutral },
                    }
                });
            }
        }
    }
    fn unsched<H: Host<Self>>(&mut self, host: &mut H, event: Self::SchedEvent) {
    }
}

impl<C: Coord> GameMinesweeper<C> {
    fn explore_tile<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, c: C) {
        if c.ring() > self.mapdata.size() {
            return;
        }
        if self.mapdata[c].owner() == u8::from(plid) {
            return;
        }
        if self.mapdata[c].owner() != u8::from(PlayerId::Neutral) {
            return;
        }
        match self.mapdata[c].item() {
            ItemKind::Safe => {
                self.capture_tile(host, plid, c);
            }
            _ => {
                self.explode_player(host, plid, c);
            }
        }
    }
    fn capture_tile<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, c: C) {
        self.mapdata[c].set_owner(u8::from(plid));
        host.msg(Plids::all(true), MwEv::Map {
            pos: c.into(),
            ev: MapEv::Owner {
                plid,
            },
        });
        self.compute_digit(host, plid, c);
    }
    fn explode_player<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, c: C) {
        match self.mapdata[c].item() {
            ItemKind::Safe => {
                return;
            },
            // minesweeper mode has no flashes, treat them as decoys
            ItemKind::Decoy | ItemKind::Flashbang => {
                host.msg(Plids::all(true), MwEv::Map {
                    pos: c.into(),
                    ev: MapEv::Item {
                        kind: ItemKind::Decoy,
                    },
                });
                host.msg(Plids::all(true), MwEv::Map {
                    pos: c.into(),
                    ev: MapEv::Explode,
                });
            },
            ItemKind::Mine => {
                host.msg(Plids::all(true), MwEv::Map {
                    pos: c.into(),
                    ev: MapEv::Item {
                        kind: ItemKind::Mine,
                    },
                });
                host.msg(Plids::all(true), MwEv::Map {
                    pos: c.into(),
                    ev: MapEv::Explode,
                });
                host.msg(Plids::all(true), MwEv::Map {
                    pos: c.into(),
                    ev: MapEv::Tile {
                        kind: TileKind::Destroyed,
                    },
                });
            },
        }

        self.mapdata[c].set_item(ItemKind::Safe);
        for c2 in c.iter_n1() {
            if let Some(tile) = self.mapdata.get(c2) {
                self.compute_digit(host, tile.owner().into(), c2);
            }
        }
    }
    fn compute_digit<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, c: C) {
        if plid == PlayerId::Neutral {
            return;
        }
        let mut digit = 0;
        let mut asterisk = false;
        for c2 in c.iter_n1() {
            if let Some(tile) = self.mapdata.get(c2) {
                if tile.owner() == u8::from(plid) {
                    // dont count our own tiles
                    continue;
                }
                if tile.item() != ItemKind::Safe {
                    digit += 1;
                }
                // minesweeper mode has no flashes, treat them as decoys
                if tile.item() == ItemKind::Decoy || tile.item() == ItemKind::Flashbang {
                    asterisk = true;
                }
            }
        }
        host.msg(Plids::from(plid), MwEv::Map {
            pos: c.into(),
            ev: MapEv::Digit {
                digit, asterisk,
            }
        });
    }
}
