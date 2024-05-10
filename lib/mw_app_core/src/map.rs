use mw_common::grid::*;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_event::<TileUpdateEvent>();
    app.configure_stage_set(
        Update,
        GridCursorSS,
        any_filter::<(Changed<GridCursor>, With<MapGovernor>)>,
    );
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct GridCursorSS;

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
