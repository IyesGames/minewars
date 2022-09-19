use mw_common::algo::FloodQ;
use mw_common::algo::FloodSelect;
use mw_common::algo::flood;
use mw_common::game::*;
use mw_common::grid::*;
use mw_common::proto::*;
use mw_common::plid::{PlayerId, PlidsSingle};

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    ExploreTile {
        tile: Pos,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SchedEvent {
    GameOverOutOfTime,
}

pub struct MwClassicSingleplayerGame<C: CompactMapCoordExt> {
    map: MapData<C, TileData>,
    max_time: Option<Duration>,
    game_over: bool,
    n_unexplored_tiles: u16,
    lives_left: u8,
    flood_q: FloodQ,
}

impl<C: CompactMapCoordExt> MwClassicSingleplayerGame<C> {
    /// Returns Err if topology does not match C
    pub fn new_with_map(data: &MapDataInitAny, max_lives: u8, max_time: Option<Duration>) -> Result<Self, ()> {
        if let Some(map) = data.map.try_get::<C>() {
            assert!(max_lives > 0);
            let mut n_unexplored_tiles = 0;

            // generate our map data representation from the worldgen init data representation
            let map = map.convert(|_, gen| {
                let mut td = TileData::default();
                if gen.kind.is_land() {
                    td.set_playable(true);
                    n_unexplored_tiles += 1;
                }
                if gen.kind == TileKind::Mountain {
                    td.set_mtn(true);
                }
                if gen.mine.is_some() {
                    td.set_mine(true);
                    n_unexplored_tiles -= 1;
                }
                td
            });

            Ok(Self {
                max_time,
                game_over: false,
                map,
                n_unexplored_tiles,
                lives_left: max_lives,
                flood_q: Default::default(),
            })
        } else {
            Err(())
        }
    }
}

impl<C: CompactMapCoordExt> Game for MwClassicSingleplayerGame<C> {
    type Plids = PlidsSingle;
    type InputAction = InputAction;
    type OutEvent = GameEvent;
    type SchedEvent = SchedEvent;

    fn init<H: Host<Self>>(&mut self, host: &mut H) {
        // schedule an event for "game over by running out of time"
        if let Some(max_time) = self.max_time {
            host.sched(Instant::now() + max_time, SchedEvent::GameOverOutOfTime);
        }
    }

    fn input_action<H: Host<Self>>(&mut self, host: &mut H, plid: PlayerId, action: Self::InputAction) {
        if self.game_over {
            return;
        }
        assert!(plid == PlayerId::from(1));
        match action {
            InputAction::ExploreTile { tile } => {
                self.explore_tile(host, tile.into());
                if self.n_unexplored_tiles == 0 {
                    // WIN!
                    host.msg(PlidsSingle, GameEvent::GameOver);
                }
            }
        }
    }

    fn unsched<H: Host<Self>>(&mut self, host: &mut H, event: Self::SchedEvent) {
        if self.game_over {
            return;
        }
        match event {
            SchedEvent::GameOverOutOfTime => {
                host.msg(PlidsSingle, GameEvent::PlayerGone {
                    plid: PlayerId::from(1),
                });
                host.msg(PlidsSingle, GameEvent::GameOver);
            }
        }
    }
}

impl<C: CompactMapCoordExt> MwClassicSingleplayerGame<C> {
    fn explore_tile<H: Host<Self>>(
        &mut self,
        host: &mut H,
        c: C,
    ) {
        if c.ring() > self.map.size() {
            // out of bounds
            return;
        }

        if !self.map[c].is_playable() {
            return;
        }

        if self.map[c].is_explored() {
            // if already explored, explore around (if safe)
            if self.map[c].digit() == 0 {
                for nc in c.iter_n0() {
                    if !self.map[nc].is_explored() {
                        self.explore_tile(host, nc);
                    }
                }
            }
            return;
        }

        if self.map[c].is_mine() {
            self.map[c].set_mine(false);
            host.msg(PlidsSingle, GameEvent::Explosion {
                tile: c.into(),
                kind: MineKind::Mine,
            });
            self.lives_left -= 1;
            if self.lives_left == 0 {
                self.game_over = true;
                host.msg(PlidsSingle, GameEvent::PlayerGone {
                    plid: PlayerId::from(1),
                });
                host.msg(PlidsSingle, GameEvent::GameOver);
            } else {
                for nc in c.iter_n1() {
                    if self.map[nc].is_explored() {
                        self.recount_digit(host, nc);
                    }
                }
            }
            return;
        }

        self.map[c].set_explored(true);
        self.n_unexplored_tiles -= 1;
        host.msg(PlidsSingle, GameEvent::Owner {
            tile: c.into(),
            owner: PlayerId::from(1),
        });

        let digit = self.recount_digit(host, c);

        // for UX / legibility: mark any nearby mountain cluster as explored
        for nc in c.iter_n0() {
            self.explore_mtncluster(host, nc);
        }

        // if safe, explore surrounding area recursively
        if digit == 0 {
            for nc in c.iter_n0() {
                if !self.map[nc].is_explored() {
                    self.explore_tile(host, nc);
                }
            }
        }
    }

    fn recount_digit<H: Host<Self>>(
        &mut self,
        host: &mut H,
        c: C,
    ) -> u8 {
        let mut digit = 0;
        for nc in c.iter_n1() {
            if self.map[nc].is_mine() {
                digit += 1;
            }
        }
        if self.map[c].digit() != digit {
            self.map[c].set_digit(digit);
            host.msg(PlidsSingle, GameEvent::Digit {
                tile: c.into(),
                digit,
            });
        }
        digit
    }

    fn explore_mtncluster<H: Host<Self>>(
        &mut self,
        host: &mut H,
        c: C,
    ) {
        if !self.map[c].is_mtn() || self.map[c].is_explored() {
            return;
        }
        self.flood_q.push_back(c.into());
        flood(&mut self.flood_q, |c2: C, _| {
            if self.map[c2].is_mtn() && !self.map[c2].is_explored() {
                self.map[c2].set_explored(true);
                host.msg(PlidsSingle, GameEvent::Owner {
                    tile: c2.into(),
                    owner: PlayerId::from(1),
                });
                FloodSelect::Yes
            } else {
                FloodSelect::No
            }
        });
    }
}

#[derive(Default)]
struct TileData(u8);

impl TileData {
    const MASK_PLAYABLE: u8  = 0b00000001;
    const SHIFT_PLAYABLE: u8 = 0;

    const MASK_OWN: u8  = 0b00000010;
    const SHIFT_OWN: u8 = 1;

    const MASK_MINE: u8  = 0b00000100;
    const SHIFT_MINE: u8 = 2;

    const MASK_MTN: u8  = 0b00010000;
    const SHIFT_MTN: u8 = 4;

    const MASK_DIGIT: u8  = 0b11100000;
    const SHIFT_DIGIT: u8 = 5;

    fn set_playable(&mut self, x: bool) {
        self.0 = self.0 & !Self::MASK_PLAYABLE | ((x as u8) << Self::SHIFT_PLAYABLE);
    }

    fn is_playable(&self) -> bool {
        self.0 & Self::MASK_PLAYABLE != 0
    }

    fn set_explored(&mut self, x: bool) {
        self.0 = self.0 & !Self::MASK_OWN | ((x as u8) << Self::SHIFT_OWN);
    }

    fn is_explored(&self) -> bool {
        self.0 & Self::MASK_OWN != 0
    }

    fn set_mine(&mut self, x: bool) {
        self.0 = self.0 & !Self::MASK_MINE | ((x as u8) << Self::SHIFT_MINE);
    }

    fn is_mine(&self) -> bool {
        self.0 & Self::MASK_MINE != 0
    }

    fn set_mtn(&mut self, x: bool) {
        self.0 = self.0 & !Self::MASK_MTN | ((x as u8) << Self::SHIFT_MTN);
    }

    fn is_mtn(&self) -> bool {
        self.0 & Self::MASK_MTN != 0
    }

    fn set_digit(&mut self, x: u8) {
        self.0 = self.0 & !Self::MASK_DIGIT | (x << Self::SHIFT_DIGIT);
    }

    fn digit(&self) -> u8 {
        (self.0 & Self::MASK_DIGIT) >> Self::SHIFT_DIGIT
    }
}
