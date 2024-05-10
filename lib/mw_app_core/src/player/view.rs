//! Per-Plid cache of game state
//!
//! A "view" is a compact representation of the game state of the entire map,
//! from the point of view of a given player. This is how the game client stores
//! the data it receives from the server, regardless of whether it is currently
//! displayed.
//!
//! This abstraction is useful in situations where the game client can view (or
//! even play as) multiple different Plids on the server. For example, when
//! spectating, you have the global spectator view, but can also switch to viewing
//! any player in the game, the way they see the world. In Playground mode, the
//! user can control any Plid. In the future, we might have gameplay that allows
//! one player to view the world as seen by another player.
//!
//! Therefore, the client needs to keep/accumulate game state for each Plid
//! that it is capable of showing to the user. In a normal game, there will be
//! only one view: that of the user's own Plid. The server will not even send
//! data for other players (to prevent cheating). But in the aforementioned
//! game modes, the client will receive data for multiple plids simultaneously,
//! and needs to retain it somehow, so that the user can switch between
//! different views when they desire.
//!
//! The game state is stored compactly, using a bit-field representation per-tile.
//! One entity per view.
//!
//! The currently active view is rendered by spawning an actual ECS representation
//! of the map. See the [`map`][mw_common::map] module for that. Those are the entities
//! that are used for actual gameplay and graphics purposes.

use mw_common::{game::{ItemKind, StructureKind, TileKind}, grid::MapDataPos, plid::PlayerId};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.configure_stage_set_no_rc(
        Update, ViewSS::Update, // TODO: RC
    );
    app.configure_stage_set_no_rc(
        Update, ViewSS::Switch, // TODO: RC
    );
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ViewSS {
    Update,
    Switch,
}

/// Bundle for a view. Insert onto the Plid entities.
#[derive(Bundle)]
pub struct ViewBundle {
    pub mapdata: ViewMapData,
}

/// The map data of a view
#[derive(Component, Clone)]
pub struct ViewMapData(pub MapDataPos<ViewTileData>);

/// Marker for entities that should be discarded on view switch.
#[derive(Component)]
pub struct DespawnOnViewSwitch;

/// Marker for entities that should be shown in a specific view and hidden in other views.
#[derive(Component)]
pub struct VisibleInView(pub PlayerId);

/// The per-tile data of a view.
///
/// This is a compact bitfield representation of the game state that needs
/// to be cached/tracked for view switching in multi-view modes.
///
/// We don't need all game state. Some can be discarded on view switch,
/// or just kept live in ECS entities and hidden (like explosion effects, etc),
/// some can be recomputed (vis levels, roads).
#[bitfield]
#[derive(Clone, Copy, Default)]
pub struct ViewTileData {
    pub owner: B4,
    pub digit: B3,
    pub asterisk: bool,
    pub kind: TileKind,
    pub item: ItemKind,
    pub has_structure: bool,
    pub structure: StructureKind,
    pub flag: B4,
    #[skip] __: B4,
}

impl ViewTileData {
    pub fn from_kind(kind: TileKind) -> Self {
        let mut t = Self::default();
        t.set_kind(kind);
        t
    }
    pub fn from_kind_item(kind: TileKind, item: ItemKind) -> Self {
        let mut t = Self::default();
        t.set_kind(kind);
        t.set_item(item);
        t
    }
}
