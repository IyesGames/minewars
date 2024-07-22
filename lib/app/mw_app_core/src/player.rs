//! Player Entities
//!
//! Player Entities represent everyone participating in a Session.
//!
//! A Session Governor must exist and its `PlayersIndex` should
//! refer to these entities.
//!
//! There should be:
//! - an entity per Plid (logical participant in a MineWars game,
//!   i.e one territory/empire).
//! - an entity per SubPlid (actual user controlling a Plid)
//! - an entity for the Spectator Plid, if spectating a game.
//!
//! During a normal gameplay session, the client is only displaying
//! the PoV of the Plid that the user is controlling (`PlidPlayingAs`
//! on the Session Governor).
//!
//! If the client should be able to display multiple PoVs of different
//! Plids, such as when spectating, then the Plid Entities should also
//! have `view` components to enable view-switcing (`PlidViewing` on
//! the Session Governor).

use mw_common::plid::PlayerId;

use crate::{prelude::*, user::UserProfile};

pub fn plugin(app: &mut App) {
}

#[derive(Bundle)]
pub struct PlayerPlidBundle {
    pub cleanup: GameFullCleanup,
    pub plid: Plid,
    pub color: PlidColor,
    pub state: PlidState,
    pub stats: PlidStats,
    pub score: PlidScore,
    pub subs: PlidSubsIndex,
}

#[derive(Bundle)]
pub struct SpectatorPlidBundle {
    pub cleanup: GameFullCleanup,
    pub plid: Plid,
}

#[derive(Bundle)]
pub struct SubPlidBundle {
    pub cleanup: GameFullCleanup,
    pub subplid: SubPlid,
    pub user_profile: SubPlidUserProfile,
    pub net_info: SubPlidNetInfo,
}

/// Marker component for plids that we are in control of
/// (many/all of them for modes like Playground, only one in normal game)
#[derive(Component)]
pub struct PlidPlayable;

#[derive(Component)]
pub struct Plid(pub PlayerId);

#[derive(Component, Default)]
pub struct PlidScore(pub u32);

#[derive(Component, Default)]
pub struct PlidStats {
    pub kills: u32,
    pub deaths: u32,
}

#[derive(Component)]
pub struct PlidColor {
    pub color: Color,
}

#[derive(Component)]
pub enum PlidState {
    /// Playing the game
    Alive,
    /// Currently in stun/timeout
    Dead {
        // Estimated respawn time (from app startup)
        end: Duration,
    },
    /// Gone from the game
    Eliminated,
}

#[derive(Component)]
pub struct PlidSubsIndex(pub Vec<Entity>);

#[derive(Component)]
pub struct SubPlid(pub u8);

#[derive(Component)]
pub struct SubPlidUserProfile(pub UserProfile);

#[derive(Component, Default)]
pub struct SubPlidNetInfo {
    pub rtt: Duration,
}

#[derive(Component)]
pub struct MatchTimeRemain {
    pub end: Duration,
}

#[derive(Component)]
pub struct NLives {
    pub lives: u8,
}

impl Default for SpectatorPlidBundle {
    fn default() -> Self {
        Self {
            cleanup: GameFullCleanup,
            plid: Plid(PlayerId::Neutral),
        }
    }
}

impl SubPlidBundle {
    pub fn new(subplid: u8, profile: &UserProfile) -> Self {
        SubPlidBundle {
            cleanup: GameFullCleanup,
            subplid: SubPlid(subplid),
            user_profile: SubPlidUserProfile(profile.clone()),
            net_info: SubPlidNetInfo::default(),
        }
    }
}

impl PlayerPlidBundle {
    pub fn new(plid: PlayerId, color: Color, subs: &[Entity]) -> Self {
        PlayerPlidBundle {
            cleanup: GameFullCleanup,
            plid: Plid(plid),
            color: PlidColor {
                color,
            },
            state: PlidState::Alive,
            stats: PlidStats::default(),
            score: PlidScore::default(),
            subs: PlidSubsIndex(subs.to_owned()),
        }
    }
}
