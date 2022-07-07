//! The Bevy Host
//!
//! Implementation of the proto::Host interface in Bevy.

use iyesengine::prelude::*;

use bevy::ecs::schedule::StateData;

use crate::{proto::{Game, Host}, app::{ActivePlid, MwLabels, AppGlobalState, StreamSource}, plid::{PlidMask, PlayerId}, HashSet};

use std::{time::Instant, marker::PhantomData, collections::BTreeMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
pub enum BevyMwHostLabels {
    /// label for the systems that drive the game
    /// *before* this, the game is untouched yet
    /// *after* this, the game will be touched no more
    DriveGame,
}

pub struct BevyMwHostPlugin<G, EvIn, EvOut, S> {
    state: S,
    pd: PhantomData<(G, EvIn, EvOut)>,
}

impl<G, EvIn, EvOut, S> BevyMwHostPlugin<G, EvIn, EvOut, S>
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Send + Sync + 'static,
    EvOut: From<(PlayerId, G::OutEvent)> + Send + Sync + 'static,
    S: StateData,
{
    pub fn new(state: S) -> Self {
        Self { state, pd: PhantomData }
    }
}

impl<G, EvIn, EvOut, S> Plugin for BevyMwHostPlugin<G, EvIn, EvOut, S>
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Send + Sync + 'static,
    EvOut: From<(PlayerId, G::OutEvent)> + Send + Sync + 'static,
    S: StateData,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyHost<G>>();
        app.add_enter_system(
            AppGlobalState::InGame,
            init_game::<G>
                .run_in_state(StreamSource::Local)
                .run_in_state(self.state.clone())
        );
        app.add_system(player_inputs::<G, EvIn>
            .run_in_state(AppGlobalState::InGame)
            .run_in_state(StreamSource::Local)
            .run_in_state(self.state.clone())
            .label(MwLabels::HostInEvents)
            .label(BevyMwHostLabels::DriveGame)
        );
        app.add_system(unscheds::<G>
            .run_in_state(AppGlobalState::InGame)
            .run_in_state(StreamSource::Local)
            .run_in_state(self.state.clone())
            .label(BevyMwHostLabels::DriveGame)
        );
        app.add_system(cancel_scheds::<G>
            .run_in_state(AppGlobalState::InGame)
            .run_in_state(StreamSource::Local)
            .run_in_state(self.state.clone())
            .after(BevyMwHostLabels::DriveGame)
        );
        app.add_system(drain_out_events::<G, EvOut>
            .run_in_state(AppGlobalState::InGame)
            .run_in_state(StreamSource::Local)
            .run_in_state(self.state.clone())
            .after(BevyMwHostLabels::DriveGame)
            .label(MwLabels::HostOutEvents)
        );
    }
}

struct BevyHost<G: Game> {
    events: Vec<(G::Plids, G::OutEvent)>,
    scheds: BTreeMap<Instant, G::SchedEvent>,
    cancel: HashSet<G::SchedEvent>,
}

impl<G: Game> Default for BevyHost<G> {
    fn default() -> Self {
        BevyHost {
            events: Vec::default(),
            scheds: BTreeMap::default(),
            cancel: HashSet::default(),
        }
    }
}

impl<G: Game> Host<G> for BevyHost<G> {
    fn msg(&mut self, plids: G::Plids, event: G::OutEvent) {
        self.events.push((plids, event));
    }
    fn sched(&mut self, time: Instant, event: G::SchedEvent) {
        self.scheds.insert(time, event);
    }
    fn desched_all(&mut self, event: G::SchedEvent) {
        self.cancel.insert(event);
    }
}

fn init_game<G>(
    mut host: ResMut<BevyHost<G>>,
    mut game: ResMut<G>,
)
where
    G: Game + Send + Sync + 'static,
{
    game.init(&mut *host);
}

fn player_inputs<G, EvIn>(
    mut host: ResMut<BevyHost<G>>,
    mut game: ResMut<G>,
    my_plid: Res<ActivePlid>,
    mut evr: EventReader<EvIn>,
)
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Send + Sync + 'static,
{
    for ev in evr.iter() {
        let action = ev.clone().into();
        game.input_action(&mut *host, my_plid.0, action);
    }
}

fn unscheds<G>(
    mut host: ResMut<BevyHost<G>>,
    mut game: ResMut<G>,
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
        game.unsched(&mut *host, ev);
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
    EvOut: From<(PlayerId, G::OutEvent)> + Send + Sync + 'static,
{
    for (plids, ev) in host.events.drain(..) {
        for plid in plids.iter(None) {
            evw.send((plid, ev.clone()).into());
        }
    }
}
