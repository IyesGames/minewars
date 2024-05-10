use crate::prelude::*;

use mw_common::{game::event::GameEvent, plid::{PlayerId, Plids}};

pub fn plugin(app: &mut App) {
    app.add_event::<GameEvent>();
    app.configure_stage_set(Update, GameOutEventSS, on_event::<GameEvent>());
    app.configure_stage_set_no_rc(Update, GameInEventSS);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GameInEventSS;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GameOutEventSS;

/// Bundle for setting up a new session (gameplay, any mode)
#[derive(Bundle)]
pub struct GameplaySessionGovernorBundle {
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
