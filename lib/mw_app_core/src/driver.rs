//! The Driver Governor
//!
//! The Driver Governor is an entity that exists if the app is in
//! some sort of live gameplay session.
//!
//! There can be various "driver implementations", meaning different
//! ways to receive game events. Some examples:
//!  - BevyDriver: Hosts an offline game by directly using our Bevy events for updating MineWars game state
//!  - NetDriver: Connects to a Host server, sends inputs and receives events via network protocol
//!  - FileDriver: Opens a MineWars file, decodes and replays events from it
//!  - ...?
//!
//! In any case, these can be different component types that live on the
//! Driver Governor entity to store their state, and their respective systems.
//!
//! The existence of a Driver Governor entity implies that the client is
//! displaying some sort of live gameplay.
//!
//! There must also exist a Session Governor. It does not make sense to
//! have a Driver without a Session.
//!
//! Normally there is also a Map Governor.

use mw_common::game::event::GameEvent;

use crate::{prelude::*, session::NeedsSessionGovernorSet};

pub fn plugin(app: &mut App) {
    app.add_event::<GameEvent>();
    app.configure_sets(Update, (
        NeedsDriverGovernorSet
            .run_if(any_with_component::<DriverGovernor>),
        NeedsGameplaySessionSet
            .in_set(NeedsDriverGovernorSet)
            .in_set(NeedsSessionGovernorSet)
    ));
    app.configure_stage_set(Update, GameOutEventSS, on_event::<GameEvent>());
    app.configure_stage_set_no_rc(Update, GameInEventSS);
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct NeedsGameplaySessionSet;

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct NeedsDriverGovernorSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GameInEventSS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GameOutEventSS;

#[derive(Component)]
pub struct DriverGovernor;
