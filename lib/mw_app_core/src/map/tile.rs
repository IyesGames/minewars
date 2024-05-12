//! All the various things we keep track of for individual map tiles.
//!
//! Every tile has its own entity to represent it.
//!
//! To find the entity for a specific Pos, look it up via the
//! `MapTileIndex` on the Map Governor.

use mw_common::{game::{CitId, ItemKind, StructureKind, TileKind}, grid::Pos, plid::PlayerId};

use crate::{view::VisibleInView, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_event::<RecomputeVisEvent>();
    app.configure_stage_set_no_rc(Update, TileUpdateSS);
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TileUpdateSS;

/// Trigger a recompute of `TileVisLevel`.
///
/// For a specific tile position, or for the whole map if None.
#[derive(Event)]
pub struct RecomputeVisEvent(pub Option<Pos>);

/// Components common to all map tiles
#[derive(Bundle)]
pub struct MapTileBundle {
    pub marker: MwMapTile,
    pub kind: TileKind,
    pub pos: MwTilePos,
}

/// Components common to all playable map tiles
#[derive(Bundle)]
pub struct PlayableTileBundle {
    pub tile: MapTileBundle,
    pub region: TileRegion,
    pub owner: TileOwner,
    pub vis: TileVisLevel,
}

/// Components of land tiles
#[derive(Bundle)]
pub struct LandTileBundle {
    pub tile: PlayableTileBundle,
    pub digit: TileDigit,
    pub gent: TileGent,
    pub roads: TileRoads,
}

/// Components of resource clusters (mountain, forest)
#[derive(Bundle)]
pub struct ResClusterTileBundle {
    pub tile: PlayableTileBundle,
}

#[derive(Bundle)]
pub struct ExplosionBundle {
    pub pos: MwTilePos,
    pub explosion: TileExplosion,
    pub view: VisibleInView,
}

/// Marker for MineWars map tile entities
#[derive(Component)]
pub struct MwMapTile;

/// Map coordinate of a given tile.
///
/// This uses our own grid coord types (Pos <-> {Hex, Sq}).
///
/// Renderer agnostic. `bevy_ecs_tilemap` `TilePos` will be added
/// by that renderer's impl.
#[derive(Component)]
pub struct MwTilePos(pub Pos);

#[derive(Component)]
pub struct TileAlert(pub Timer);

/// Map region (cit association) of a tile
#[derive(Component)]
pub struct TileRegion(pub u8);

/// Plid who owns the tile
#[derive(Component)]
pub struct TileOwner(pub PlayerId);

/// Any minesweeper digit to be displayed on the tile.
///
/// The `u8` is the digit value (`0` means no digit).
/// The `bool` is whether to display an asterisk.
#[derive(Component)]
pub struct TileDigit(pub u8, pub bool);

/// Any Road connections to neighboring tiles.
///
/// If the tile has no road, this is zero.
///
/// Otherwise, the value is a bitmask with a bit representing
/// each adjacent tile that also has a road.
///
/// This representation allows efficiently rendering roads correctly.
#[derive(Component)]
pub struct TileRoads(pub u8);

/// Is there any "game entity" on a land tile?
#[derive(Component)]
pub enum TileGent {
    /// Tile has nothing on it
    Empty,
    /// Tile contains a City
    Cit(CitId),
    /// Tile contains an item
    Item(ItemKind),
    /// Tile contains a non-road structure
    /// (ignore roads, represent them using `TileRoads` instead)
    Structure(StructureKind),
    /// Tile contains a Flag (placed by the given player)
    Flag(PlayerId),
}

/// Visibility level of the given tile
#[derive(Component)]
pub enum TileVisLevel {
    Fog,
    Visible,
}

#[derive(Component)]
pub struct TileExplosion {
    pub e: Entity,
    pub item: Option<ItemKind>,
}
