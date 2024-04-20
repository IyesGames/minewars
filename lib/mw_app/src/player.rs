use mw_common::plid::PlayerId;

use crate::prelude::*;

/// Marker component for plids that we are in control of
/// (many/all of them for modes like Playground, only one in normal game)
#[derive(Component)]
pub struct PlidPlayable;

/// The plid that the user controls. This is not necessarily the same
/// as `PlidViewing`.
#[derive(Resource)]
pub struct PlidPlayingAs(pub PlayerId);

#[derive(Resource)]
pub struct PlayersIndex(pub Vec<Entity>);

#[derive(Component)]
pub struct PlayerPlid(pub PlayerId);

#[derive(Component)]
pub struct PlayerDisplayName(pub String);

// #[derive(Component)]
// pub struct PlayerStats {
//     pub kills: u32,
//     pub deaths: u32,
// }

#[derive(Component)]
pub enum PlayerState {
    /// Playing the game
    Alive,
    /// Currently in stun/timeout
    Dead,
    /// Gone from the game
    Eliminated,
    /// Inactive and frozen (such as on disconnects)
    Protected,
}

// #[derive(Component)]
// pub struct PlayerOwnsCits(pub u32);

#[derive(Bundle)]
pub struct PlayerBundle {
    pub plid: PlayerPlid,
    pub name: PlayerDisplayName,
    pub state: PlayerState,
    // pub stats: PlayerStats,
    // pub cits: PlayerOwnsCits,
}

#[derive(Bundle)]
pub struct SpectatorPlidBundle {
    plid: PlayerPlid,
}

impl Default for SpectatorPlidBundle {
    fn default() -> Self {
        Self {
            plid: PlayerPlid(PlayerId::Neutral),
        }
    }
}
