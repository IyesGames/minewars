//! The Bevy Host
//!
//! Allows offline gameplay without a server.

use crate::GameOutEventSS;
use crate::player::PlayersIndex;
use crate::player::PlidPlayingAs;
use crate::prelude::*;

use mw_common::driver::*;
use mw_common::plid::*;

use std::collections::BTreeMap;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BevyHostSS;

pub fn plugin<G, EvIn, EvOut>(app: &mut App)
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Event,
    EvOut: From<(PlayerId, G::OutEvent)> + Clone + Event,
{
    app.configure_stage_set_no_rc(Update, BevyHostSS);
    app.add_systems(
        OnEnter(SessionKind::BevyHost),
        init::<G>
            .run_if(resource_exists::<BevyHost<G>>)
    );
    app.add_systems(Update, (
        (
            player_inputs::<G, EvIn>
                .in_set(SetStage::WantChanged(GameInEventSS)),
            unscheds::<G>
                .run_if(rc_unscheds::<G>),
        ),
        (
            cancel_scheds::<G>,
            drain_out_events::<G, EvOut>
                .in_set(SetStage::Provide(GameOutEventSS)),
            game_over::<G>
                .after(drain_out_events::<G, EvOut>),
        ),
    ).chain()
        .in_set(InStateSet(AppState::InGame))
        .in_set(InStateSet(SessionKind::BevyHost))
        .run_if(resource_exists::<BevyHost<G>>)
        .in_set(SetStage::Provide(BevyHostSS))
    );
}

#[derive(Resource)]
pub struct BevyHost<G: Game> {
    game: G,
    state: BevyHostState<G>,
}

struct BevyHostState<G: Game> {
    events: Vec<(Plids, G::OutEvent)>,
    scheds: BTreeMap<Instant, G::SchedEvent>,
    cancel: HashSet<G::SchedEvent>,
    init_data: Option<Box<G::InitData>>,
    game_over: bool,
}

impl<G: Game> BevyHost<G> {
    pub fn new(game: G, init_data: G::InitData) -> Self {
        BevyHost {
            game,
            state: BevyHostState {
                events: Vec::default(),
                scheds: BTreeMap::default(),
                cancel: HashSet::default(),
                init_data: Some(Box::new(init_data)),
                game_over: false,
            },
        }
    }
}

impl<G: Game> Host<G> for BevyHostState<G> {
    fn msg(&mut self, plids: Plids, event: G::OutEvent) {
        self.events.push((plids, event));
    }
    fn sched(&mut self, time: Instant, event: G::SchedEvent) {
        self.scheds.insert(time, event);
    }
    fn desched_all(&mut self, event: G::SchedEvent) {
        self.cancel.insert(event);
    }
    fn game_over(&mut self) {
        self.game_over = true;
    }
}

fn init<G>(
    host: ResMut<BevyHost<G>>,
)
where
    G: Game + Send + Sync + 'static,
{
    let host = host.into_inner();
    let init_data = host.state.init_data.take().unwrap();
    host.game.init(&mut host.state, *init_data);
}

fn player_inputs<G, EvIn>(
    host: ResMut<BevyHost<G>>,
    my_plid: Res<PlidPlayingAs>,
    mut evr: EventReader<EvIn>,
)
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Event,
{
    let host = host.into_inner();
    for ev in evr.read() {
        let action = ev.clone().into();
        host.game.input(&mut host.state, my_plid.0, action);
    }
}

fn rc_unscheds<G>(
    host: Option<Res<BevyHost<G>>>,
) -> bool
where
    G: Game + Send + Sync + 'static,
{
    if let Some(host) = host {
        !host.state.scheds.is_empty()
    } else {
        false
    }
}

fn unscheds<G>(
    mut host: ResMut<BevyHost<G>>,
)
where
    G: Game + Send + Sync + 'static,
{
    if host.state.scheds.is_empty() {
        return;
    }

    let now = Instant::now();
    let mut split = host.state.scheds.split_off(&now);
    std::mem::swap(&mut split, &mut host.state.scheds);

    let host = host.into_inner();
    for ev in split.into_values() {
        host.game.unsched(&mut host.state, ev);
    }
}

fn cancel_scheds<G>(
    mut host: ResMut<BevyHost<G>>,
    mut temp: Local<Vec<Instant>>,
)
where
    G: Game + Send + Sync + 'static,
{
    // PERF: replace with `drain_filter` when in stable Rust
    // remove local Vec, avoid doing 2 passes
    temp.clear();
    for (k, ev) in host.state.scheds.iter() {
        if host.state.cancel.contains(ev) {
            temp.push(*k);
        }
    }
    for k in temp.drain(..) {
        host.state.scheds.remove(&k);
    }
    host.state.cancel.clear();
}

fn drain_out_events<G, EvOut>(
    mut host: ResMut<BevyHost<G>>,
    players: Option<Res<PlayersIndex>>,
    mut evw: EventWriter<EvOut>,
)
where
    G: Game + Send + Sync + 'static,
    EvOut: From<(PlayerId, G::OutEvent)> + Event,
{
    let n_plids = players.map(|p| p.0.len() as u8 - 1);
    for (plids, ev) in host.state.events.drain(..) {
        for plid in plids.iter(n_plids) {
            evw.send((plid, ev.clone()).into());
        }
    }
}

fn game_over<G>(
    mut commands: Commands,
    host: Res<BevyHost<G>>,
)
where
    G: Game + Send + Sync + 'static,
{
    if host.state.game_over {
        commands.remove_resource::<BevyHost<G>>();
    }
}

