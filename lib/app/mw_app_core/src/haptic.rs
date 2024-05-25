use mw_common::grid::Pos;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_event::<HapticEvent>();
    app.configure_stage_set(Update, HapticEventSS, on_event::<HapticEvent>());
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct HapticEventSS;

/// Events to trigger different kinds of shake/haptic effects.
///
/// Different systems can handle these events to implement the
/// appropriate effects via camera shake, hardware vibrators, etc.
#[derive(Event, Debug)]
pub struct HapticEvent {
    /// Grid/map position, if known
    pub pos: Option<Pos>,
    /// What causes the haptic?
    pub kind: HapticEventKind,
}

/// The different things that can cause haptic feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(Serialize, Deserialize)]
pub enum HapticEventKind {
    /// Ambience / background explosions
    BackgroundTremor,

    /// Explosion on player territory
    ExplosionOurTerritory,
    /// Explosion outside of player territory
    ExplosionForeignTerritory,

    /// Strike on player's mine
    ExplosionTheyDestroyOurMine,
    /// Enemy stepped on our mine
    ExplosionMineKill,
    /// Player stepped on a mine
    ExplosionMineDeath,
    /// Enemy stepped on someone else's mine
    ExplosionSomeoneDied,

    /// Player structure destroyed
    StructureDestroyedOur,
    /// Player destroyed a structure
    StructureDestroyedTheir,
}
