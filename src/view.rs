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

use bevy::render::extract_resource::ExtractResource;
use mw_common::{plid::PlayerId, game::{ItemKind, TileKind, MapDescriptor, StructureKind}, grid::{Coord, MapData, Topology, Hex, Sq}};
use modular_bitfield::prelude::*;

use crate::prelude::*;
use crate::map::*;

pub struct GameViewPlugin;

impl Plugin for GameViewPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(
            Update,
            ViewSwitchSet.run_if(resource_exists_and_changed::<PlidViewing>())
        );
        app.add_systems(Update, (
            switch_view_despawn,
            switch_view_showhide,
            (
                switch_view_update_map_tiles::<Hex>,
                switch_view_update_map_digits::<Hex>,
                switch_view_update_map_gents::<Hex>,
                switch_view_update_map_roads::<Hex>,
            ).in_set(MapTopologySet(Topology::Hex)),
            (
                switch_view_update_map_tiles::<Sq>,
                switch_view_update_map_digits::<Sq>,
                switch_view_update_map_gents::<Sq>,
                switch_view_update_map_roads::<Sq>,
            ).in_set(MapTopologySet(Topology::Sq)),
        ).in_set(ViewSwitchSet));
    }
}

/// All systems that run whenever a view switch is triggered, are in this set.
///
/// Systems that want to change the view should run before. Systems that need
/// a valid map should run after.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewSwitchSet;

/// Index of all the views in the current gameplay session.
#[derive(Resource, ExtractResource, Clone)]
pub struct Views(Vec<Entity>);

/// The currently active view -- what is displayed to the user.
#[derive(Resource, ExtractResource, Clone)]
pub struct PlidViewing(PlayerId);

/// The plid that the user controls. This is not necessarily the same
/// as `PlidViewing`.
#[derive(Resource)]
pub struct PlidPlayingAs(PlayerId);

/// The per-tile data of a view.
///
/// This is a compact bitfield representation of the game state that needs
/// to be cached/tracked for view switching in multi-view modes.
///
/// Since it is stored for all plids in the session, we don't want to waste memory.
///
/// We don't need all game state. Some can be discarded on view switch,
/// or just kept live in ECS entities and hidden (like explosion effects, etc),
/// some can be recomputed (vis levels, roads).
#[bitfield]
#[derive(Clone, Copy)]
pub struct ViewTileData {
    pub owner: B4,
    pub digit: B3,
    pub kind: TileKind,
    pub item: ItemKind,
    pub has_structure: bool,
    pub structure: StructureKind,
    #[skip] __: B1,
}

/// The map data of a view
#[derive(Component, Clone)]
pub struct ViewMapData<C: Coord>(MapData<C, ViewTileData>);

/// Bundle for a view entity
#[derive(Bundle)]
pub struct ViewBundle<C: Coord> {
    plid: PlayerId,
    mapdata: ViewMapData<C>,
}

/// Marker for entities that should be discarded on view switch.
#[derive(Component)]
pub struct DespawnOnViewSwitch;

/// Marker for entities that should be shown in a specific view and hidden in other views.
#[derive(Component)]
pub struct VisibleInView(PlayerId);

fn switch_view_despawn(
    mut commands: Commands,
    q: Query<Entity, With<DespawnOnViewSwitch>>,
) {
    for e in &q {
        commands.entity(e).despawn_recursive();
    }
}

fn switch_view_showhide(
    viewing: Res<PlidViewing>,
    mut q: Query<(&mut Visibility, &VisibleInView)>,
) {
    for (mut vis, viewvis) in &mut q {
        if viewvis.0 == viewing.0 {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn switch_view_update_map_tiles<C: Coord>(
    viewing: Res<PlidViewing>,
    views: Res<Views>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileKind, &mut TileOwner)>,
    mut evw_visrecompute: EventWriter<RecomputeVisEvent>,
) {
    evw_visrecompute.send(RecomputeVisEvent(None));

    let e_view = views.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    for (pos, mut kind, mut owner) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        let tiledata = &viewdata.0[c];
        *kind = tiledata.kind();
        owner.0 = tiledata.owner().into();
    }
}

fn switch_view_update_map_digits<C: Coord>(
    viewing: Res<PlidViewing>,
    views: Res<Views>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileDigit)>,
) {
    let e_view = views.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    for (pos, mut digit) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        let tiledata = &viewdata.0[c];
        digit.0 = tiledata.owner().into();
    }
}

fn switch_view_update_map_gents<C: Coord>(
    viewing: Res<PlidViewing>,
    views: Res<Views>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileGent)>,
) {
    let e_view = views.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    for (pos, mut gent) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        let tiledata = &viewdata.0[c];
        // TODO:
        let is_cit = false;
        *gent = if is_cit {
            // TODO:
            TileGent::Cit(0)
        } else if tiledata.has_structure() {
            TileGent::Structure(tiledata.structure())
        } else {
            let item = tiledata.item();
            if item != ItemKind::Safe {
                TileGent::Item(item)
            } else {
                TileGent::Empty
            }
        };
    }
}

fn switch_view_update_map_roads<C: Coord>(
    viewing: Res<PlidViewing>,
    views: Res<Views>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileRoads)>,
) {
    let e_view = views.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    for (pos, mut roads) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        roads.0 = if viewdata.0[c].structure() == StructureKind::Road {
            viewdata.0.get_ringmask(c, |d| d.structure() == StructureKind::Road)
        } else {
            0
        };
    }
}
