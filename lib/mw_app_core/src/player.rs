use mw_common::plid::PlayerId;

use crate::prelude::*;

pub mod view;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        view::plugin,
    ));
}

#[derive(Bundle)]
pub struct PlayerPlidBundle {
    pub plid: Plid,
    pub color: PlidColor,
    pub state: PlidState,
    pub stats: PlidStats,
    pub subs: PlidSubsIndex,
}

#[derive(Bundle)]
pub struct SpectatorPlidBundle {
    pub plid: Plid,
}

#[derive(Bundle)]
pub struct SubPlidBundle {
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

#[derive(Component)]
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
pub struct SubPlidUserProfile {
    pub display_name: String,
}

#[derive(Component)]
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
            plid: Plid(PlayerId::Neutral),
        }
    }
}
