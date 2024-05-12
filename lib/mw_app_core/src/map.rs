//! The Map Governor
//!
//! The Map Governor is an entity that exists if the app has known
//! map data that it currently loaded into it.
//!
//! To implement a simple map viewer, or a map editor, it is enough
//! to only have a Map Governor, no Session or Driver Governors.
//!
//! To implement a gameplay state, there should also be a Session
//! Governor, and also a Driver Governor while gameplay is live.
//!
//! The Map Governor carries info about the loaded map, such as
//! its properties, index of tile entities, current cursor position,
//! and a copy of the initial "pristine" map data before any gameplay.

use mw_common::grid::*;

use crate::prelude::*;

pub mod cit;
pub mod tile;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        cit::plugin,
        tile::plugin,
    ));
    app.add_event::<TileUpdateEvent>();
    app.configure_sets(Update, (
        NeedsMapGovernorSet
            .run_if(any_with_component::<MapGovernor>),
    ));
    app.configure_stage_set(
        Update,
        GridCursorSS,
        any_filter::<(Changed<GridCursor>, With<MapGovernor>)>,
    );
    for topo in enum_iterator::all::<Topology>() {
        app.configure_sets(Update, MapTopologySet(topo).run_if(map_topology_is(topo)));
    }
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct NeedsMapGovernorSet;

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct GridCursorSS;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MapTopologySet(pub Topology);

#[derive(Event)]
pub struct TileUpdateEvent {
    pub pos: Pos,
    pub entity: Entity,
}

#[derive(Bundle)]
pub struct MapGovernorBundle {
    pub marker: MapGovernor,
    pub desc: MapDescriptor,
    pub tile_index: MapTileIndex,
    pub cit_index: CitIndex,
    pub grid_cursor: GridCursor,
    pub grid_cursor_tile_entity: GridCursorTileEntity,
}

/// Marker component for the map governor entity
#[derive(Component)]
pub struct MapGovernor;

#[derive(Component)]
pub struct MapDescriptor {
    pub size: u8,
    pub topology: Topology,
}

#[derive(Component)]
pub struct MapTileIndex(pub MapDataPos<Entity>);

#[derive(Component)]
pub struct CitIndex {
    pub by_pos: HashMap<Pos, Entity>,
    pub by_id: Vec<Entity>,
}

#[derive(Component, Default)]
pub struct GridCursor(pub Option<Pos>);

#[derive(Component, Default)]
pub struct GridCursorTileEntity(pub Option<Entity>);

pub fn map_topology_is(topo: Topology) -> impl FnMut(Query<&MapDescriptor>) -> bool {
    move |q: Query<&MapDescriptor>| {
        q.get_single().ok()
            .map(|desc| desc.topology == topo)
            .unwrap_or(false)
    }
}
