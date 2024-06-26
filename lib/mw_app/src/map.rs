//! ECS representation of MineWars map state
//!
//! These are the entities that represent the map tiles of the view that
//! is currently activated.

use mw_common::grid::*;
use mw_common::plid::*;
use mw_common::game::*;

use crate::prelude::*;
use crate::view::VisibleInView;

mod update;

pub fn plugin(app: &mut App) {
    app.add_plugins(update::plugin);
    app.add_event::<RecomputeVisEvent>();
    app.init_resource::<GridCursorTileEntity>();
    app.init_resource::<TileUpdateQueue>();
    for topo in enum_iterator::all::<Topology>() {
        app.configure_sets(Update, MapTopologySet(topo).run_if(map_topology_is(topo)));
        app.configure_sets(Update, NeedsMapSet.run_if(resource_exists::<MapDescriptor>));
    }
}

pub fn map_topology_is(topo: Topology) -> impl FnMut(Option<Res<MapDescriptor>>) -> bool {
    move |desc: Option<Res<MapDescriptor>>| {
        desc.map(|desc| desc.topology == topo).unwrap_or(false)
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapTopologySet(pub Topology);

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NeedsMapSet;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TilemapSetupSet;

#[derive(Resource)]
pub struct TilemapInitted;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapUpdateSet {
    TileKind,
    TileOwner,
    TileDigit,
    TileGent,
    TileRoads,
}

#[derive(Resource, Default)]
pub struct GridCursorTileEntity(pub Option<Entity>);

#[derive(Resource)]
pub struct MapTileIndex(pub MapDataPos<Entity>);

#[derive(Resource)]
pub struct ItemIndex(pub HashMap<Pos, Entity>);

#[derive(Resource)]
pub struct CitIndex {
    pub by_pos: HashMap<Pos, Entity>,
    pub by_id: Vec<Entity>,
}

/// Keeps track of tile entities whose map data has changed.
///
/// This is used as a more efficient alternative to Bevy's change detection,
/// because iterating queries with changed filters still requires accessing
/// all entities.
#[derive(Resource, Default)]
pub struct TileUpdateQueue(Vec<Entity>);

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
    Limited,
    Full,
}

pub enum TileExplosionKind {
    Normal,
    Decoy,
}

#[derive(Component)]
pub struct TileExplosion(pub Entity, pub TileExplosionKind);

/// Components common to all map tiles
#[derive(Bundle)]
pub struct MapTileBundle {
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

/// Trigger a recompute of `TileVisLevel`.
///
/// For a specific tile position, or for the whole map if None.
#[derive(Event)]
pub struct RecomputeVisEvent(pub Option<Pos>);

#[derive(Bundle)]
pub struct CitBundle {
    pub regid: CitRegion,
    pub owner: CitOwner,
    pub res: CitRes,
}

#[derive(Component)]
pub struct CitRegion(pub u8);

#[derive(Component)]
pub struct CitOwner(pub PlayerId);

#[derive(Component)]
pub struct CitRes {
    pub money: u32,
    pub income: u16,
    pub res: u16,
}

impl TileUpdateQueue {
    pub fn clear(&mut self) {
        self.0.clear()
    }
    pub fn is_marked_all(&self) -> bool {
        self.0.get(0) != Some(&Entity::from_bits(u64::MAX))
    }
    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }
    pub fn mark_all(&mut self) {
        // add a sentinel value to indicate all tiles to be checked
        self.0.clear();
        self.0.push(Entity::from_bits(u64::MAX));
    }
    pub fn mark_one(&mut self, entity: Entity) {
        // if we are already checking all tiles, no need to do anything
        if self.is_marked_all() {
            self.0.push(entity);
        }
    }
    pub fn mark_coord(&mut self, index: &MapTileIndex, c: Pos) {
        // if we are already checking all tiles, no need to do anything
        if self.is_marked_all() {
            if let Some(e) = index.0.get(c) {
                self.0.push(*e);
            }
        }
    }
}

/// Helper code to setup all the map-related ECS stuff.
///
/// This is not a standalone system, because we can have map data
/// that comes from different sources (server, file, procgen) and
/// we want to be able to initialize the tilemap from any of them.
pub fn setup_map<C: Coord, D, L: MapDataLayout<C>>(
    world: &mut World,
    mapdata: &MapData<C, D, L>,
    cits: &[Pos],
    f_tilekind: impl Fn(&D) -> TileKind,
    f_regid: impl Fn(&D) -> u8,
) {
    let mut tile_index = MapTileIndex(
        MapDataPos::new(mapdata.size(), Entity::PLACEHOLDER)
    );

    let mut cit_index = CitIndex {
        by_id: Vec::with_capacity(cits.len()),
        by_pos: Default::default(),
    };

    let item_index = ItemIndex(Default::default());

    for (c, d) in mapdata.iter() {
        let tilekind = f_tilekind(d);
        let b_base = MapTileBundle {
            kind: tilekind,
            pos: MwTilePos(c.into()),
        };
        let e_tile = if tilekind.ownable() {
            let b_playable = PlayableTileBundle {
                tile: b_base,
                region: TileRegion(f_regid(d)),
                owner: TileOwner(PlayerId::Neutral),
                vis: TileVisLevel::Full,
            };
            if tilekind.is_land() {
                world.spawn(LandTileBundle {
                    tile: b_playable,
                    digit: TileDigit(0, false),
                    gent: TileGent::Empty,
                    roads: TileRoads(0),
                }).id()
            } else if tilekind.is_rescluster() {
                world.spawn(ResClusterTileBundle {
                    tile: b_playable,
                }).id()
            } else {
                world.spawn(b_playable).id()
            }
        } else {
            world.spawn(b_base).id()
        };
        tile_index.0[c.into()] = e_tile;
    }

    for (i, cit_pos) in cits.iter().enumerate() {
        let cit_pos = *cit_pos;
        let e_cit = world.spawn(
            CitBundle {
                regid: CitRegion(i as u8),
                owner: CitOwner(PlayerId::Neutral),
                res: CitRes {
                    money: 0,
                    income: 0,
                    res: 0,
                },
            },
        ).id();
        cit_index.by_id.push(e_cit);
        cit_index.by_pos.insert(cit_pos, e_cit);
        world.entity_mut(tile_index.0[(cit_pos).into()])
            .insert(TileGent::Cit(i as u8));
    }

    world.insert_resource(tile_index);
    world.insert_resource(cit_index);
    world.insert_resource(item_index);
    world.insert_resource(MapDescriptor {
        size: mapdata.size(),
        topology: C::TOPOLOGY,
    });
}
