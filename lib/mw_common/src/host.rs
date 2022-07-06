//! The Bevy Host
//!
//! Implementation of the proto::Host interface in Bevy.

use iyesengine::prelude::*;

use crate::{proto::{Game, Host}, app::ActivePlid, plid::{PlidMask, PlayerId}, HashSet};

use std::{time::Instant, marker::PhantomData, collections::BTreeMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
pub enum BevyMwHostLabels {
    /// anything feeding input events for the game should come *before*
    InEvents,
    /// anything needing output events from the game should come *after*
    OutEvents,
    /// label for the systems that drive the game
    /// *before* this, the game is untouched yet
    /// *after* this, the game will be touched no more
    DriveGame,
}

pub struct BevyMwHostPlugin<G, EvIn, EvOut> {
    pd: PhantomData<(G, EvIn, EvOut)>,
}

impl<G, EvIn, EvOut> BevyMwHostPlugin<G, EvIn, EvOut>
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Send + Sync + 'static,
    EvOut: From<(PlayerId, G::OutEvent)> + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self { pd: PhantomData }
    }
}

impl<G, EvIn, EvOut> Plugin for BevyMwHostPlugin<G, EvIn, EvOut>
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Send + Sync + 'static,
    EvOut: From<(PlayerId, G::OutEvent)> + Send + Sync + 'static,
{
    fn build(&self, app: &mut App) {
        app.init_resource::<BevyHost<G>>();
        app.add_system(player_inputs::<G, EvIn>
            .label(BevyMwHostLabels::InEvents)
            .label(BevyMwHostLabels::DriveGame)
        );
        app.add_system(unscheds::<G>
            .label(BevyMwHostLabels::DriveGame)
        );
        app.add_system(cancel_scheds::<G>
            .after(BevyMwHostLabels::DriveGame)
        );
        app.add_system(drain_out_events::<G, EvOut>
            .after(BevyMwHostLabels::DriveGame)
            .label(BevyMwHostLabels::OutEvents)
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

fn player_inputs<G, EvIn>(
    mut host: ResMut<BevyHost<G>>,
    game: Option<ResMut<G>>,
    my_plid: Res<ActivePlid>,
    mut evr: EventReader<EvIn>,
)
where
    G: Game + Send + Sync + 'static,
    EvIn: Into<G::InputAction> + Clone + Send + Sync + 'static,
{
    if let Some(mut game) = game {
        for ev in evr.iter() {
            let action = ev.clone().into();
            game.input_action(&mut *host, my_plid.0, action);
        }
    }
}

fn unscheds<G>(
    mut host: ResMut<BevyHost<G>>,
    game: Option<ResMut<G>>,
)
where
    G: Game + Send + Sync + 'static,
{
    if let Some(mut game) = game {
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
