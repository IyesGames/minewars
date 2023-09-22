//! The Bevy Host
//!
//! Allows offline gameplay without a server.

use crate::player::PlidPlayingAs;
use crate::prelude::*;

use mw_common::driver::*;
use mw_common::plid::*;

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemSet)]
pub enum BevyHostSet {
    All,
    Game,
    PostGame,
    EvIn,
    EvOut,
}

pub struct BevyMwHostPlugin<G, EvIn, EvOut> {
    pd: PhantomData<(G, EvIn, EvOut)>,
}

impl<G, EvIn, EvOut> BevyMwHostPlugin<G, EvIn, EvOut>
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Event,
    EvOut: From<(PlayerId, G::OutEvent)> + Clone + Event,
{
    pub fn new() -> Self {
        Self { pd: PhantomData }
    }
}

impl<G, EvIn, EvOut> Plugin for BevyMwHostPlugin<G, EvIn, EvOut>
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Event,
    EvOut: From<(PlayerId, G::OutEvent)> + Clone + Event,
{
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            (
                player_inputs::<G, EvIn>.in_set(BevyHostSet::EvIn),
                unscheds::<G>,
            ).in_set(BevyHostSet::Game),
            (
                cancel_scheds::<G>,
            ).in_set(BevyHostSet::PostGame),
            drain_out_events::<G, EvOut>.in_set(BevyHostSet::EvOut),
        ).in_set(BevyHostSet::All)
         .run_if(in_state(AppState::InGame))
         .run_if(in_state(SessionKind::BevyHost))
         .run_if(resource_exists::<BevyHost<G>>())
         .run_if(resource_exists::<BevyGame<G>>())
        );
    }
}

#[derive(Resource)]
pub struct BevyGame<G: Game>(pub G);

#[derive(Resource)]
struct BevyHost<G: Game> {
    events: Vec<(Plids, G::OutEvent)>,
    scheds: BTreeMap<Instant, G::SchedEvent>,
    cancel: HashSet<G::SchedEvent>,
}

impl<G: Game> BevyHost<G> {
    pub fn new() -> Self {
        BevyHost {
            events: Vec::default(),
            scheds: BTreeMap::default(),
            cancel: HashSet::default(),
        }
    }
}

impl<G: Game> Host<G> for BevyHost<G> {
    fn msg(&mut self, plids: Plids, event: G::OutEvent) {
        self.events.push((plids, event));
    }
    fn sched(&mut self, time: Instant, event: G::SchedEvent) {
        self.scheds.insert(time, event);
    }
    fn desched_all(&mut self, event: G::SchedEvent) {
        self.cancel.insert(event);
    }
}

fn player_inputs<G, EvIn>(
    mut host: ResMut<BevyHost<G>>,
    mut game: ResMut<BevyGame<G>>,
    my_plid: Res<PlidPlayingAs>,
    mut evr: EventReader<EvIn>,
)
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Event,
{
    for ev in evr.iter() {
        let action = ev.clone().into();
        game.0.input(&mut *host, my_plid.0, action);
    }
}

fn unscheds<G>(
    mut host: ResMut<BevyHost<G>>,
    mut game: ResMut<BevyGame<G>>,
)
where
    G: Game + Send + Sync + 'static,
{
    if host.scheds.is_empty() {
        return;
    }

    let now = Instant::now();
    let mut split = host.scheds.split_off(&now);
    std::mem::swap(&mut split, &mut host.scheds);
    for ev in split.into_values() {
        game.0.unsched(&mut *host, ev);
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
    for (k, ev) in host.scheds.iter() {
        if host.cancel.contains(ev) {
            temp.push(*k);
        }
    }
    for k in temp.drain(..) {
        host.scheds.remove(&k);
    }
}

fn drain_out_events<G, EvOut>(
    mut host: ResMut<BevyHost<G>>,
    mut evw: EventWriter<EvOut>,
)
where
    G: Game + Send + Sync + 'static,
    EvOut: From<(PlayerId, G::OutEvent)> + Event,
{
    for (plids, ev) in host.events.drain(..) {
        for plid in plids.iter(None) {
            evw.send((plid, ev.clone()).into());
        }
    }
}

