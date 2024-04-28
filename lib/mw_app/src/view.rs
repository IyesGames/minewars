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
use modular_bitfield::prelude::*;

use mw_common::grid::*;
use mw_common::plid::*;
use mw_common::game::*;

use crate::player::PlidPlayable;
use crate::player::PlidPlayingAs;
use crate::prelude::*;
use crate::map::*;
use crate::player::PlayersIndex;

mod update;

pub fn plugin(app: &mut App) {
    app.add_plugins(update::plugin);
    app.configure_sets(
        Update,
        ViewSwitchSet.run_if(resource_exists_and_changed::<PlidViewing>)
    );
    app.add_systems(Update, (
        kbd_viewswitch,
    ).before(ViewSwitchSet).in_set(NeedsMapSet));
    app.add_systems(Update, (
        switch_view_despawn,
        switch_view_showhide,
        (
            switch_view_update_map_tilekind::<Hex>.in_set(MapUpdateSet::TileKind),
            switch_view_update_map_owners::<Hex>.in_set(MapUpdateSet::TileOwner),
            switch_view_update_map_digits::<Hex>.in_set(MapUpdateSet::TileDigit),
            switch_view_update_map_gents::<Hex>.in_set(MapUpdateSet::TileGent),
            switch_view_update_map_roads::<Hex>.in_set(MapUpdateSet::TileRoads),
        ).in_set(MapTopologySet(Topology::Hex)),
        (
            switch_view_update_map_tilekind::<Sq>.in_set(MapUpdateSet::TileKind),
            switch_view_update_map_owners::<Sq>.in_set(MapUpdateSet::TileOwner),
            switch_view_update_map_digits::<Sq>.in_set(MapUpdateSet::TileDigit),
            switch_view_update_map_gents::<Sq>.in_set(MapUpdateSet::TileGent),
            switch_view_update_map_roads::<Sq>.in_set(MapUpdateSet::TileRoads),
        ).in_set(MapTopologySet(Topology::Sq)),
    ).in_set(ViewSwitchSet));
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewUpdateSet;

/// All systems that run whenever a view switch is triggered, are in this set.
///
/// Systems that want to change the view should run before. Systems that need
/// a valid map should run after.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ViewSwitchSet;

/// The currently active view -- what is displayed to the user.
#[derive(Resource, ExtractResource, Clone)]
pub struct PlidViewing(pub PlayerId);

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

/// The map data of a view
#[derive(Component, Clone)]
pub struct ViewMapData<C: Coord>(pub MapData<C, ViewTileData>);

/// Bundle for a view entity
#[derive(Bundle)]
pub struct ViewBundle<C: Coord> {
    pub mapdata: ViewMapData<C>,
}

/// Marker for entities that should be discarded on view switch.
#[derive(Component)]
pub struct DespawnOnViewSwitch;

/// Marker for entities that should be shown in a specific view and hidden in other views.
#[derive(Component)]
pub struct VisibleInView(pub PlayerId);

impl ViewTileData {
    pub fn from_kind(kind: TileKind) -> Self {
        let mut t = Self::default();
        t.set_kind(kind);
        t
    }
}

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

fn switch_view_update_map_tilekind<C: Coord>(
    viewing: Res<PlidViewing>,
    players: Res<PlayersIndex>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileKind)>,
) {
    let e_view = players.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    // TODO: handle any need for bundle/component changes due to tile kind
    for (pos, mut kind) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        let tiledata = &viewdata.0[c];
        *kind = tiledata.kind();
    }
}

fn switch_view_update_map_owners<C: Coord>(
    viewing: Res<PlidViewing>,
    players: Res<PlayersIndex>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileOwner)>,
    mut evw_visrecompute: EventWriter<RecomputeVisEvent>,
) {
    evw_visrecompute.send(RecomputeVisEvent(None));

    let e_view = players.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    for (pos, mut owner) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        let tiledata = &viewdata.0[c];
        owner.0 = tiledata.owner().into();
    }
}

fn switch_view_update_map_digits<C: Coord>(
    viewing: Res<PlidViewing>,
    players: Res<PlayersIndex>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileDigit)>,
) {
    let e_view = players.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    for (pos, mut digit) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        let tiledata = &viewdata.0[c];
        digit.0 = tiledata.digit();
        digit.1 = tiledata.asterisk();
    }
}

fn switch_view_update_map_gents<C: Coord>(
    viewing: Res<PlidViewing>,
    players: Res<PlayersIndex>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileGent)>,
) {
    let e_view = players.0[viewing.0.i()];
    let Ok(viewdata) = q_view.get(e_view) else {
        error!("View for {:?} does not exist!", viewing.0);
        return;
    };
    for (pos, mut gent) in q_maptile.iter_mut() {
        let c: C = pos.0.into();
        let tiledata = &viewdata.0[c];
        // preserve CITS, they are special
        if let TileGent::Cit(_) = *gent {
            continue;
        }
        let item = tiledata.item();
        let flag = PlayerId::from(tiledata.flag());
        *gent = if tiledata.has_structure() {
            TileGent::Structure(tiledata.structure())
        } else if flag != PlayerId::Neutral {
            TileGent::Flag(flag)
        } else if item != ItemKind::Safe {
            TileGent::Item(item)
        } else {
            TileGent::Empty
        };
    }
}

fn switch_view_update_map_roads<C: Coord>(
    viewing: Res<PlidViewing>,
    players: Res<PlayersIndex>,
    q_view: Query<&ViewMapData<C>>,
    mut q_maptile: Query<(&MwTilePos, &mut TileRoads)>,
) {
    let e_view = players.0[viewing.0.i()];
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

fn kbd_viewswitch(
    kbd: Res<ButtonInput<KeyCode>>,
    mut viewing: ResMut<PlidViewing>,
    mut playingas: ResMut<PlidPlayingAs>,
    players: Res<PlayersIndex>,
    q_view: Query<(), Or<(With<ViewMapData<Hex>>, With<ViewMapData<Sq>>)>>,
    q_plid: Query<(), With<PlidPlayable>>,
) {
    let mut newplid = if kbd.just_pressed(KeyCode::F1) {
        PlayerId::from(1)
    } else if kbd.just_pressed(KeyCode::F2) {
        PlayerId::from(2)
    } else if kbd.just_pressed(KeyCode::F3) {
        PlayerId::from(3)
    } else if kbd.just_pressed(KeyCode::F4) {
        PlayerId::from(4)
    } else if kbd.just_pressed(KeyCode::F5) {
        PlayerId::from(5)
    } else if kbd.just_pressed(KeyCode::F6) {
        PlayerId::from(6)
    } else if kbd.just_pressed(KeyCode::F7) {
        PlayerId::from(7)
    } else if kbd.just_pressed(KeyCode::F8) {
        PlayerId::from(8)
    } else if kbd.just_pressed(KeyCode::F9) {
        PlayerId::from(9)
    } else if kbd.just_pressed(KeyCode::F10) {
        PlayerId::from(10)
    } else if kbd.just_pressed(KeyCode::F11) {
        PlayerId::from(11)
    } else if kbd.just_pressed(KeyCode::F12) {
        PlayerId::from(12)
    } else {
        return;
    };

    if kbd.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
        // check if the plid exists and is controllable
        let Some(e_plid) = players.0.get(newplid.i()) else {
            return;
        };
        if q_plid.get(*e_plid).is_ok() {
            playingas.0 = newplid;
            info!("Playing as {:?}", newplid);
        }
    } else {
        // Toggle to spectator if pressing the key of the current plid
        if viewing.0 == newplid {
            newplid = PlayerId::Neutral;
        }
        // check if the plid and view actually exist
        let Some(e_plid) = players.0.get(newplid.i()) else {
            return;
        };
        if q_view.get(*e_plid).is_ok() {
            viewing.0 = newplid;
            info!("Viewing {:?}", newplid);
        }
    }
}
