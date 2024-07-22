//! The Session Governor
//!
//! The Session Governor is an entity that exists if the app is
//! displaying some sort of MineWars game session.
//!
//! It carries information about players and the rules/features
//! of the game. Different game modes can be implemented via
//! additional/optional components on the Session Governor.
//!
//! If the session is actually live, there should also exist
//! a Driver Governor, to process events.
//!
//! Normally there is also a Map Governor.

use crate::prelude::*;

use mw_common::plid::{PlayerId, Plids};

pub fn plugin(app: &mut App) {
    app.configure_sets(Update, (
        NeedsSessionGovernorSet
            .run_if(any_with_component::<SessionGovernor>),
    ));
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct NeedsSessionGovernorSet;

/// Bundle for setting up a new session (gameplay, any mode)
#[derive(Bundle)]
pub struct SessionGovernorBundle {
    pub cleanup: GameFullCleanup,
    pub marker: SessionGovernor,
    pub playing_as: PlidPlayingAs,
    pub viewing: PlidViewing,
    pub players: PlayersIndex,
}

/// Marker component for the session governor entity
#[derive(Component)]
pub struct SessionGovernor;

/// The plid that the user controls
/// This is not necessarily the same as `PlidViewing`.
#[derive(Component)]
pub struct PlidPlayingAs(pub PlayerId);

/// The plid whose PoV is being rendered
#[derive(Component)]
pub struct PlidViewing(pub PlayerId);

/// Player info entities associated with the session.
#[derive(Component)]
pub struct PlayersIndex {
    pub plids: Plids,
    pub e_plid: Vec<Entity>,
    pub e_subplid: Vec<Vec<Entity>>,
}

/// Scoring based on how many cits a plid owns
#[derive(Component)]
pub struct PlidScoreByCits;

/// Scoring based on % of map owned
#[derive(Component)]
pub struct PlidScoreByOwnedPct;

impl SessionGovernorBundle {
    pub fn new(my_plid: PlayerId, e_plid: &[Entity], e_subplid: &[&[Entity]]) -> Self {
        let plidsmask = (1 << (e_plid.len() + 1)) - 1;
        SessionGovernorBundle {
            cleanup: GameFullCleanup,
            marker: SessionGovernor,
            playing_as: PlidPlayingAs(my_plid),
            viewing: PlidViewing(my_plid),
            players: PlayersIndex {
                plids: Plids(plidsmask as u16),
                e_plid: e_plid.to_owned(),
                e_subplid: e_subplid.iter()
                    .map(|x| Vec::from(*x))
                    .collect(),
            },
        }
    }
}
