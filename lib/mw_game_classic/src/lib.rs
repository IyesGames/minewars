use mw_common::game::*;
use mw_common::grid::*;
use mw_common::proto::*;
use mw_common::plid::{PlayerId, PlidsSingle};

use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FlagState {
    None,
    Flag,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputAction {
    ExploreTileSingle {
        tile: Pos,
    },
    ExploreTileGreedy {
        tile: Pos,
    },
    Flag {
        tile: Pos,
        state: FlagState,
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
}

impl<C: CompactMapCoordExt> MwClassicSingleplayerGame<C> {
    pub fn new_with_map(data: MapDataInit<C>, max_time: Option<Duration>) -> Self {
        Self {
            max_time,
            game_over: false,
            map: data.map.convert(|c, gen| {
                let td = TileData::default();
                // TODO
                td
            }),
        }
    }
}

impl<C: CompactMapCoordExt> Game for MwClassicSingleplayerGame<C> {
    type Plids = PlidsSingle;
    type InputAction = InputAction;
    type OutEvent = GameEvent;
    type SchedEvent = SchedEvent;

    fn init<H: Host<Self>>(&mut self, host: &mut H) {
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
            InputAction::ExploreTileSingle { tile } => {
                host.msg(PlidsSingle, GameEvent::Owner {
                    tile, owner: plid,
                });
            }
            InputAction::ExploreTileGreedy { tile } => {
                unimplemented!()
            }
            InputAction::Flag { tile, state } => {
                unimplemented!()
            }
        }
    }

    fn unsched<H: Host<Self>>(&mut self, host: &mut H, event: Self::SchedEvent) {
        if self.game_over {
            return;
        }
        match event {
            SchedEvent::GameOverOutOfTime => {
                host.msg(PlidsSingle, GameEvent::GameOver {
                    plid: PlayerId::from(1),
                })
            }
        }
    }
}

#[derive(Default)]
struct TileData(u8);
