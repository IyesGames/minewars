use std::collections::BTreeMap;

use async_channel::{Receiver, Sender, TryRecvError};
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use mw_app_core::{driver::{DriverGovernor, GameOutEventSS, NeedsDriverGovernorSet}, session::{NeedsSessionGovernorSet, PlidPlayingAs, SessionGovernor}};
use mw_common::driver::*;

use crate::prelude::*;

pub struct OfflineHostPlugin<G: Game, EIn, EOut> {
    _pd: PhantomData<(G, EIn, EOut)>,
}

impl<G: Game, EIn, EOut> OfflineHostPlugin<G, EIn, EOut>
where
    EIn: Event + Clone + Into<<G::Io as GameIo>::InputAction>,
    EOut: Event + From<(Plids, <G::Io as GameIo>::OutEvent)>,
{
    pub fn new() -> Self {
        Self {
            _pd: PhantomData,
        }
    }
}

impl<G: Game, EIn, EOut> Plugin for OfflineHostPlugin<G, EIn, EOut>
where
    EIn: Event + Clone + Into<<G::Io as GameIo>::InputAction>,
    EOut: Event + From<(Plids, <G::Io as GameIo>::OutEvent)>,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update,
            init_offline_game::<G>
                .track_progress()
                .in_set(InStateSet(AppState::GameLoading))
                .in_set(NeedsSessionGovernorSet)
                .in_set(NeedsDriverGovernorSet)
                .run_if(any_filter::<(With<OfflineHost<G>>, With<DriverGovernor>)>)
        );
        app.add_systems(Update,
            update_offline_game::<G, EIn, EOut>
                .in_set(InStateSet(AppState::InGame))
                .in_set(SetStage::Provide(GameOutEventSS))
                .in_set(NeedsSessionGovernorSet)
                .in_set(NeedsDriverGovernorSet)
                .run_if(any_filter::<(With<OfflineHost<G>>, With<DriverGovernor>)>)
        );
    }
}

#[derive(Component)]
pub struct OfflineHost<G: Game>(OfflineHostState<G>);

enum OfflineHostState<G: Game> {
    NotInitialized {
        game: G,
        init_data: Box<G::InitData>,
    },
    Initializing {
        task: Task<(G, HostState<G>)>,
    },
    Initialized {
        game: G,
        host: HostState<G>,
    },
    Running {
        tx: Sender<TaskIn<G>>,
        rx: Receiver<TaskOut<G>>,
        task: Task<()>,
        next_sched: Option<Instant>,
    },
    GameOver,
}

enum TaskIn<G: Game> {
    GameInput {
        plid: PlayerId,
        event: <G::Io as GameIo>::InputAction,
    },
    SchedTrigger(Instant),
    Maintain,
}

enum TaskOut<G: Game> {
    GameOutput(GameOutput<G::Io>),
    NextSched(Instant),
    NoSchedsRemain,
    GameOver,
}

impl<G: Game> OfflineHost<G> {
    pub fn new(game: G, init_data: Box<G::InitData>) -> Self {
        Self(OfflineHostState::NotInitialized { game, init_data })
    }
}

fn init_offline_game<G: Game>(
    mut q_driver: Query<&mut OfflineHost<G>, With<DriverGovernor>>,
) -> Progress {
    let state = &mut q_driver.single_mut().0;
    let temp = std::mem::replace(state, OfflineHostState::GameOver);
    let r;
    *state = match temp {
        OfflineHostState::NotInitialized { mut game, init_data } => {
            let rt = AsyncComputeTaskPool::get();
            let mut host = HostState::<G>::default();
            let task = rt.spawn(async move {
                game.init(&mut host, init_data);
                (game, host)
            });
            info!("Offline game initializing.");
            r = false.into();
            OfflineHostState::Initializing { task }
        }
        OfflineHostState::Initializing { mut task } => {
            if let Some((game, host)) = block_on(poll_once(&mut task)) {
                info!("Offline game initialized.");
                r = true.into();
                OfflineHostState::Initialized { game, host }
            } else {
                r = false.into();
                OfflineHostState::Initializing { task }
            }
        }
        s => {
            r = true.into();
            s
        }
    };
    r
}

fn update_offline_game<G: Game, EIn, EOut>(
    q_session: Query<&PlidPlayingAs, With<SessionGovernor>>,
    mut q_driver: Query<&mut OfflineHost<G>, With<DriverGovernor>>,
    mut evr_in: EventReader<EIn>,
    mut evw_out: EventWriter<EOut>,
)
where
    EIn: Event + Clone + Into<<G::Io as GameIo>::InputAction>,
    EOut: Event + From<(Plids, <G::Io as GameIo>::OutEvent)>,
{
    let plid = q_session.single().0;
    let state = &mut q_driver.single_mut().0;
    let temp = std::mem::replace(state, OfflineHostState::GameOver);
    *state = match temp {
        OfflineHostState::GameOver => OfflineHostState::GameOver,
        OfflineHostState::Running { tx, rx, task, mut next_sched } => {
            let mut game_over = false;
            if evr_in.is_empty() {
                if let Some(time) = &next_sched {
                    let now = Instant::now();
                    if now >= *time {
                        if tx.try_send(TaskIn::SchedTrigger(now)).is_err() {
                            game_over = true;
                        }
                        next_sched = None;
                    }
                }
            }
            for ev in evr_in.read() {
                if tx.try_send(TaskIn::GameInput { plid, event: ev.clone().into()}).is_err() {
                    game_over = true;
                }
            }
            loop {
                match rx.try_recv() {
                    Ok(out) => match out {
                        TaskOut::GameOutput(out) => {
                            evw_out.send((out.plids, out.output).into());
                        }
                        TaskOut::NoSchedsRemain => {
                            next_sched = None;
                        }
                        TaskOut::NextSched(time)  => {
                            next_sched = Some(time);
                        }
                        TaskOut::GameOver => {
                            game_over = true;
                        }
                    }
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                    Err(TryRecvError::Closed) => {
                        game_over = true;
                        break;
                    }
                }
            }
            if game_over {
                OfflineHostState::GameOver
            } else {
                OfflineHostState::Running {
                    tx, rx, task, next_sched,
                }
            }
        }
        OfflineHostState::Initialized { game, host } => {
            let rt = AsyncComputeTaskPool::get();
            let (tx_in, rx_in) = async_channel::unbounded();
            let (tx_out, rx_out) = async_channel::unbounded();
            let task = rt.spawn(async move {
                task_host_game(game, host, rx_in, tx_out).await;
            });
            tx_in.try_send(TaskIn::Maintain).ok();
            OfflineHostState::Running {
                tx: tx_in,
                rx: rx_out,
                task,
                next_sched: None,
            }
        }
        _ => panic!("Offline Game Not Initialized!")
    }
}

struct HostState<G: Game> {
    events: Vec<GameOutput<G::Io>>,
    scheds: BTreeMap<Instant, <G::Io as GameIo>::SchedEvent>,
    cancel: HashSet<<G::Io as GameIo>::SchedEvent>,
    game_over: bool,
}

impl<G: Game> Default for HostState<G> {
    fn default() -> Self {
        Self {
            events: default(),
            scheds: default(),
            cancel: default(),
            game_over: false,
        }
    }
}

impl<G: Game> Host<G::Io> for HostState<G> {
    fn msg(&mut self, output: GameOutput<G::Io>) {
        self.events.push(output);
    }
    fn sched(&mut self, time: Instant, event: <G::Io as GameIo>::SchedEvent) {
        self.scheds.insert(time, event);
    }
    fn desched_all(&mut self, event: <G::Io as GameIo>::SchedEvent) {
        self.cancel.insert(event);
    }
    fn game_over(&mut self) {
        self.game_over = true;
    }
}

impl<G: Game> HostState<G> {
    fn maintain_scheds(&mut self) -> TaskOut<G> {
        if !self.cancel.is_empty() {
            self.scheds
                .extract_if(|_, ev| self.cancel.contains(ev))
                .count();
            self.cancel.clear();
        }
        if let Some((time, _)) = self.scheds.first_key_value() {
            TaskOut::NextSched(*time)
        } else {
            TaskOut::NoSchedsRemain
        }
    }
}

async fn task_host_game<G: Game>(
    mut game: G,
    mut host: HostState<G>,
    rx: Receiver<TaskIn<G>>,
    tx: Sender<TaskOut<G>>,
) {
    info!("Offline Game Host started.");
    'main: while let Ok(ev) = rx.recv().await {
        match ev {
            TaskIn::SchedTrigger(time) => {
                let mut split = host.scheds.split_off(&time);
                std::mem::swap(&mut split, &mut host.scheds);
                for ev in split.into_values() {
                    game.unsched(&mut host, ev);
                    for out in host.events.drain(..) {
                        let Ok(_) = tx.send(TaskOut::GameOutput(out)).await else {
                            break 'main;
                        };
                    }
                    if host.game_over {
                        let _ = tx.send(TaskOut::GameOver).await;
                        break 'main;
                    }
                    while game.needs_maintain() {
                        game.maintain();
                    }
                }
                let Ok(_) = tx.send(host.maintain_scheds()).await else {
                    break 'main;
                };
            }
            TaskIn::GameInput { plid, event }  => {
                let input = GameInput {
                    plid,
                    subplid: 0, // FIXME
                    input: event,
                };
                game.input(&mut host, input);
                for out in host.events.drain(..) {
                    let Ok(_) = tx.send(TaskOut::GameOutput(out)).await else {
                        break 'main;
                    };
                }
                if host.game_over {
                    let _ = tx.send(TaskOut::GameOver).await;
                    break 'main;
                }
                let Ok(_) = tx.send(host.maintain_scheds()).await else {
                    break 'main;
                };
                while game.needs_maintain() {
                    game.maintain();
                }
            }
            TaskIn::Maintain => {
                for out in host.events.drain(..) {
                    let Ok(_) = tx.send(TaskOut::GameOutput (out)).await else {
                        break 'main;
                    };
                }
                if host.game_over {
                    let _ = tx.send(TaskOut::GameOver).await;
                    break 'main;
                }
                let Ok(_) = tx.send(host.maintain_scheds()).await else {
                    break 'main;
                };
                while game.needs_maintain() {
                    game.maintain();
                }
            }
        }
    }
    rx.close();
    tx.close();
    info!("Offline Game Host shutting down.")
}
