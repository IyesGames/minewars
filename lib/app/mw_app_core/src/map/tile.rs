//! All the various things we keep track of for individual map tiles.
//!
//! Every tile has its own entity to represent it.
//!
//! To find the entity for a specific Pos, look it up via the
//! `MapTileIndex` on the Map Governor.

use mw_common::{game::{CitId, ItemKind, MwDigit, StructureKind, TileKind}, grid::Pos, plid::PlayerId};

use crate::{view::VisibleInView, prelude::*};

use super::MapGovernor;

pub fn plugin(app: &mut App) {
    app.configure_stage_set(Update, TileUpdateSS, rc_updated_tiles);
    app.configure_sets(Update, (
        TileUpdateSet::External
            .in_set(SetStage::Provide(TileUpdateSS)),
        TileUpdateSet::Internal
            .after(TileUpdateSet::External)
            .in_set(SetStage::Provide(TileUpdateSS)),
    ));
}

/// Anything that updates components on map entities
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct TileUpdateSS;

/// Sets to help organize systems that update components on map entities
#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum TileUpdateSet {
    /// Systems that handle updates based on incoming game events and other such "external" sources
    External,
    /// Systems that update tiles based on local / client-side features
    Internal,
}

/// Used to avoid Bevy change detection overhead.
#[derive(Component, Default)]
pub struct TileUpdateQueue(pub Option<TilesToUpdate>);

pub enum TilesToUpdate {
    All,
    Specific(Vec<Entity>),
}

/// Components common to all map tiles
#[derive(Bundle)]
pub struct MapTileBundle {
    pub cleanup: GamePartialCleanup,
    pub marker: MwMapTile,
    pub kind: TileKind,
    pub pos: MwTilePos,
}

/// Components common to all playable map tiles
#[derive(Bundle)]
pub struct PlayableTileBundle {
    pub region: TileRegion,
    pub owner: TileOwner,
    pub vis: TileVisLevel,
}

/// Components of land tiles
#[derive(Bundle, Default)]
pub struct LandTileBundle {
    pub digit_internal: TileDigitInternal,
    pub digit_external: TileDigitExternal,
    pub gent: TileGent,
    pub roads: TileRoads,
}

/// Components of resource clusters (mountain, forest)
#[derive(Bundle)]
pub struct ResClusterTileBundle {
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

/// Any digit that we are told to display.
///
/// This comes from game updates.
#[derive(Component, Default)]
pub struct TileDigitInternal(pub MwDigit);

/// Any digit that we compute locally based on known item locations.
///
/// This is the "preview" of what digits are
/// expected to look like for other players.
#[derive(Component, Default)]
pub struct TileDigitExternal(pub MwDigit);

/// Any Road connections to neighboring tiles.
///
/// If the tile has no road, this is zero.
///
/// Otherwise, the value is a bitmask with a bit representing
/// each adjacent tile that also has a road.
///
/// This representation allows efficiently rendering roads correctly.
#[derive(Component, Default)]
pub struct TileRoads(pub u8);

/// Is there any "game entity" on a land tile?
#[derive(Component, Default)]
pub enum TileGent {
    /// Tile has nothing on it
    #[default]
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

fn rc_updated_tiles(
    q_map: Query<&TileUpdateQueue, With<MapGovernor>>,
) -> bool {
    q_map.get_single()
        .map(|tuq| tuq.0.is_some())
        .unwrap_or(false)
}

impl TileUpdateQueue {
    pub fn queue_one(&mut self, e: Entity) {
        match &mut self.0 {
            None => {
                self.0 = Some(TilesToUpdate::Specific(vec![e]));
            }
            Some(TilesToUpdate::All) => {},
            Some(TilesToUpdate::Specific(vec)) => {
                vec.push(e);
            }
        }
    }
    pub fn queue_all(&mut self) {
        self.0 = Some(TilesToUpdate::All);
    }
    pub fn clear(&mut self) {
        self.0 = None;
    }
}
