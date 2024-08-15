use mw_common::prelude::*;
use mw_common::algo::*;
use mw_common::driver::*;

use modular_bitfield::prelude::*;

pub mod minegen;

pub mod builder;

/// Settings that can be configured for a session of the Minesweeper game mode
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "bevy", derive(Reflect))]
#[serde(default)]
pub struct MinesweeperSettings {
    /// Each plid will get eliminated from the game when they step on a mine this many times.
    pub n_lives: u8,
    /// If nonzero, limit the maximum time allowed for the game.
    pub time_limit_secs: u16,
}

impl Default for MinesweeperSettings {
    fn default() -> Self {
        Self {
            n_lives: 1,
            time_limit_secs: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MinesweeperInitData {
    pub minegen: minegen::MineGenSettings,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "bevy", derive(Event))]
pub enum MinesweeperInputAction {
    ExploreTile {
        pos: Pos,
    },
    ToggleFlag {
        pos: Pos,
    },
}

#[derive(Clone, Copy)]
pub struct PlayerData {
    n_owned: u16,
    n_lives: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinesweeperSchedEvent {
    GameOverOutOfTime,
}

type MyRng = rand_pcg::Pcg64Mcg;

pub struct MinesweeperIo;

impl GameIo for MinesweeperIo {
    type InputAction = MinesweeperInputAction;
    type OutEvent = MwEv;
    type SchedEvent = MinesweeperSchedEvent;
}

pub enum GameMinesweeper {
    Hex(GameMinesweeperTopo<Hex>),
    Sq(GameMinesweeperTopo<Sq>),
}

impl Game for Box<GameMinesweeper> {
    type Io = MinesweeperIo;
    type InitData = MinesweeperInitData;

    fn init<H: Host<MinesweeperIo>>(&mut self, host: &mut H, initdata: Box<Self::InitData>) {
        match self.as_mut() {
            GameMinesweeper::Hex(game) => game.init(host, initdata),
            GameMinesweeper::Sq(game) => game.init(host, initdata),
        }
    }
    fn input<H: Host<MinesweeperIo>>(&mut self, host: &mut H, input: GameInput<MinesweeperIo>) {
        match self.as_mut() {
            GameMinesweeper::Hex(game) => game.input(host, input),
            GameMinesweeper::Sq(game) => game.input(host, input),
        }
    }
    fn unsched<H: Host<MinesweeperIo>>(&mut self, host: &mut H, event: MinesweeperSchedEvent) {
        match self.as_mut() {
            GameMinesweeper::Hex(game) => game.unsched(host, event),
            GameMinesweeper::Sq(game) => game.unsched(host, event),
        }
    }
    fn unreliable<H: Host<MinesweeperIo>>(&mut self, host: &mut H) {
        match self.as_mut() {
            GameMinesweeper::Hex(game) => game.unreliable(host),
            GameMinesweeper::Sq(game) => game.unreliable(host),
        }
    }
}

pub struct GameMinesweeperTopo<C: Coord> {
    settings: MinesweeperSettings,
    mapdata: MapDataC<C, TileData>,
    playerdata: Vec<PlayerData>,
    n_unexplored_tiles: u16,
    floodq: FloodQ,
    rng: MyRng,
}

#[bitfield]
#[derive(Clone, Copy, Default)]
struct TileData {
    owner: B4,
    flag: B4,
    item: ItemKind,
    kind: TileKind,
    #[skip] __: B3,
}

impl<C: Coord> Game for GameMinesweeperTopo<C> {
    type Io = MinesweeperIo;
    type InitData = MinesweeperInitData;

    fn init<H: Host<MinesweeperIo>>(&mut self, host: &mut H, initdata: Box<MinesweeperInitData>) {
        minegen::gen_mines(
            &initdata.minegen,
            &mut self.mapdata,
            &mut self.rng,
            |d| d.kind(),
            |d, i| d.set_item(i),
        );
        // schedule an event for "game over by running out of time"
        if self.settings.time_limit_secs != 0 {
            host.msg((Plids::all(true), MwEv::Player {
                plid: PlayerId::Neutral,
                subplid: None,
                ev: PlayerEv::MatchTimeRemain {
                    secs: self.settings.time_limit_secs,
                },
            }).into());
            host.sched(
                Instant::now() + Duration::from_secs(self.settings.time_limit_secs as u64),
                MinesweeperSchedEvent::GameOverOutOfTime
            );
        }
    }
    fn input<H: Host<MinesweeperIo>>(&mut self, host: &mut H, input: GameInput<MinesweeperIo>) {
        if u8::from(input.plid) > self.playerdata.len() as u8 || input.plid == PlayerId::Neutral {
            return;
        }
        if let Some(playerdata) = self.playerdata.get(input.plid.i()-1) {
            if playerdata.n_lives == 0 {
                return;
            }
        } else {
            return;
        }
        match input.input {
            MinesweeperInputAction::ExploreTile { pos } => {
                self.explore_tile(host, input.plid, pos.into());
            }
            MinesweeperInputAction::ToggleFlag { pos } => {
                self.flag(host, input.plid, pos.into());
            }
        }
    }
    fn unsched<H: Host<MinesweeperIo>>(&mut self, host: &mut H, event: MinesweeperSchedEvent) {
        match event {
            MinesweeperSchedEvent::GameOverOutOfTime => {
                for (i, playerdata) in self.playerdata.iter().enumerate() {
                    if playerdata.n_lives > 0 {
                        host.msg((Plids::all(true), MwEv::Player {
                            plid: PlayerId::from(i as u8 + 1),
                            subplid: None,
                            ev: PlayerEv::Eliminated,
                        }).into());
                    }
                }
                host.game_over();
            }
        }
    }
}

impl<C: Coord> GameMinesweeperTopo<C> {
    fn flag<H: Host<MinesweeperIo>>(&mut self, host: &mut H, plid: PlayerId, c: C) {
        if c.ring() > self.mapdata.size() {
            return;
        }
        if !self.mapdata[c].kind().is_land() || self.mapdata[c].owner() != 0 {
            return;
        }
        if self.mapdata[c].flag() == 0 {
            if c.iter_n1().any(|c2| self.mapdata[c2].owner() == u8::from(plid)) {
                self.mapdata[c].set_flag(u8::from(plid));
                host.msg((Plids::all(true), MwEv::Flag {
                    plid, pos: c.into()
                }).into());
            }
        } else if self.mapdata[c].flag() == u8::from(plid) {
            self.mapdata[c].set_flag(0);
            host.msg((Plids::all(true), MwEv::Flag {
                plid: PlayerId::Neutral, pos: c.into()
            }).into());
        }
    }
    fn explore_tile<H: Host<MinesweeperIo>>(&mut self, host: &mut H, plid: PlayerId, c: C) {
        if c.ring() > self.mapdata.size() {
            return;
        }

        if !self.mapdata[c].kind().is_land() {
            return;
        }

        // if this is the player's first tile, and it is not adjacent to another player's
        // territory, guarantee it to be safe (forgive any mine)
        if let Some(playerdata) = self.playerdata.get(plid.i()-1) {
            if playerdata.n_owned == 0 {
                if self.mapdata[c].item() == ItemKind::Mine {
                    if c.iter_n1().all(|c2| self.mapdata[c2].owner() == 0) {
                        self.mapdata[c].set_item(ItemKind::Safe);
                    }
                }
            }
        }

        let owner = self.mapdata[c].owner();

        if owner == u8::from(plid) {
            let (digit, asterisk) = self.compute_digit(plid, c);
            if digit == 0 {
                for c2 in c.iter_n1() {
                    if self.mapdata[c2].owner() == 0 {
                        self.capture_tile(host, plid, c2, true);
                    }
                }
            }
            if digit == 1 && asterisk {
                for c2 in c.iter_n1() {
                    if self.mapdata[c2].owner() == 0 {
                        match self.mapdata[c2].item() {
                            ItemKind::Safe => {
                                self.capture_tile(host, plid, c2, false);
                            }
                            _ => {
                                self.explode_player(host, plid, c2);
                            }
                        }
                    }
                }
            }
        } else if owner == 0 {
            match self.mapdata[c].item() {
                ItemKind::Safe => {
                    self.capture_tile(host, plid, c, true);
                }
                _ => {
                    self.explode_player(host, plid, c);
                }
            }
        }
    }
    fn capture_tile<H: Host<MinesweeperIo>>(&mut self, host: &mut H, plid: PlayerId, mut c: C, recurse: bool) {
        let mut q = vec![];
        loop {
            if !self.mapdata[c].kind().is_land() {
                break;
            }
            if let Some(playerdata) = self.playerdata.get_mut(plid.i()-1) {
                playerdata.n_owned += 1;
            }
            self.n_unexplored_tiles -= 1;
            self.mapdata[c].set_owner(u8::from(plid));
            if self.mapdata[c].flag() != 0 {
                self.mapdata[c].set_flag(0);
                host.msg((Plids::all(true), MwEv::Flag {
                    plid: PlayerId::Neutral, pos: c.into()
                }).into());
            }
            host.msg((Plids::all(true), MwEv::TileOwner {
                plid, pos: c.into()
            }).into());
            let digit = self.compute_send_digit(host, plid, c);
            for c2 in c.iter_n1() {
                let kind = self.mapdata[c2].kind();
                if kind.is_rescluster() {
                    self.mapdata[c2].set_owner(u8::from(plid));
                    host.msg((Plids::all(true), MwEv::TileOwner {
                        plid, pos: c2.into()
                    }).into());
                    self.floodq.clear();
                    self.floodq.push_back(c2.into());
                    flood(&mut self.floodq, |c3, _| {
                        if self.mapdata[c3].kind() == kind && self.mapdata[c3].owner() != u8::from(plid) {
                            self.mapdata[c3].set_owner(u8::from(plid));
                            host.msg((Plids::all(true), MwEv::TileOwner {
                                plid, pos: c3.into()
                            }).into());
                            FloodSelect::Yes
                        } else {
                            FloodSelect::No
                        }
                    });
                }
                if recurse && kind.is_land() && digit.0 == 0 &&self.mapdata[c2].owner() == 0 {
                    q.push(c2);
                }
            }
            if let Some(next_c) = q.pop() {
                c = next_c;
            } else {
                break;
            }
        }
        if self.n_unexplored_tiles == 0 {
            host.game_over();
        }
    }
    fn explode_player<H: Host<MinesweeperIo>>(&mut self, host: &mut H, plid: PlayerId, c: C) {
        let mut capture = true;
        match self.mapdata[c].item() {
            ItemKind::Safe => {
                return;
            },
            // minesweeper mode has no flashes, treat them as decoys
            ItemKind::Decoy | ItemKind::Trap => {
                if self.mapdata[c].flag() != 0 {
                    self.mapdata[c].set_flag(0);
                    host.msg((Plids::all(true), MwEv::Flag {
                        plid: PlayerId::Neutral, pos: c.into(),
                    }).into());
                }
                host.msg((Plids::all(true), MwEv::RevealItem {
                    item: ItemKind::Decoy, pos: c.into(),
                }).into());
                host.msg((Plids::all(true), MwEv::Explode {
                    pos: c.into(),
                }).into());
            },
            ItemKind::Mine => {
                // we now have an extra safe/explorable tile
                self.n_unexplored_tiles += 1;
                if self.mapdata[c].flag() != 0 {
                    self.mapdata[c].set_flag(0);
                    host.msg((Plids::all(true), MwEv::Flag {
                        plid: PlayerId::Neutral, pos: c.into(),
                    }).into());
                }
                host.msg((Plids::all(true), MwEv::RevealItem {
                    item: ItemKind::Mine, pos: c.into(),
                }).into());
                host.msg((Plids::all(true), MwEv::Explode {
                    pos: c.into(),
                }).into());
                host.msg((Plids::all(true), MwEv::TileKind {
                    kind: TileKind::Destroyed, pos: c.into(),
                }).into());
                if let Some(playerdata) = self.playerdata.get_mut(plid.i()-1) {
                    if playerdata.n_lives > 0 {
                        playerdata.n_lives -= 1;
                    }
                    if playerdata.n_lives == 0 {
                        host.msg((Plids::all(true), MwEv::Player {
                            plid,
                            subplid: None,
                            ev: PlayerEv::Eliminated,
                        }).into());
                        capture = false;
                    }
                    host.msg((Plids::all(true), MwEv::Player {
                        plid,
                        subplid: None,
                        ev: PlayerEv::LivesRemain {
                            lives: playerdata.n_lives,
                        },
                    }).into());
                    if self.playerdata.iter().all(|p| p.n_lives == 0) {
                        host.game_over();
                    }
                }
            },
        }

        self.mapdata[c].set_item(ItemKind::Safe);
        for c2 in c.iter_n1() {
            if let Some(tile) = self.mapdata.get(c2) {
                self.compute_send_digit(host, tile.owner().into(), c2);
            }
        }
        // tile is now safe, why not autocapture it
        if capture {
            self.capture_tile(host, plid, c, false);
        }
    }
    fn compute_digit(&mut self, plid: PlayerId, c: C) -> (u8, bool) {
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
                if tile.item() == ItemKind::Decoy || tile.item() == ItemKind::Trap {
                    asterisk = true;
                }
            }
        }
        (digit, asterisk)
    }
    fn compute_send_digit<H: Host<MinesweeperIo>>(&mut self, host: &mut H, plid: PlayerId, c: C) -> (u8, bool) {
        let (digit, asterisk) = self.compute_digit(plid, c);
        host.msg((Plids::from(plid), MwEv::DigitCapture {
            pos: c.into(), digit: MwDigit {
                digit, asterisk,
            },
        }).into());
        (digit, asterisk)
    }
}
