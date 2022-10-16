use crate::prelude::*;

use mw_common::grid::map::CompactMapCoordExt;
use mw_common::game::{ProdState, MineKind, MapDataInitAny, TileKind, MapDescriptor, GameParams, CitId};
use mw_common::plid::PlayerId;
use mw_common::grid::*;

use crate::AppGlobalState;

mod gfx2d;

#[derive(Debug, PartialEq, Eq)]
pub enum MwMapGfxBackend {
    Sprites,
    Tilemap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
pub enum MapLabels {
    /// Anything that sends MapEvents should come *before*
    ApplyEvents,
    /// Anything relying on valid TileOwner should come *after*
    TileOwner,
    /// Anything relying on valid TileDigit should come *after*
    TileDigit,
    /// Anything relying on valid TileVisible should come *after*
    TileVisible,
    /// Anything relying on valid TileMine should come *after*
    TileMine,
    /// Anything relying on up-to-date cit entities should come *after*
    CitUpdate,
}

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MapEvent>();
        app.add_system(
            setup_map
                .track_progress()
                .run_in_state(AppGlobalState::GameLoading)
        );
        app.add_exit_system(AppGlobalState::InGame, despawn_with_recursive::<MapCleanup>);
        app.add_exit_system(AppGlobalState::InGame, remove_resource::<MapDescriptor>);
        app.add_exit_system(AppGlobalState::InGame, remove_resource::<TileEntityIndex>);
        app.add_exit_system(AppGlobalState::InGame, remove_resource::<MineIndex>);
        app.add_system(map_event_owner
            .run_in_state(AppGlobalState::InGame)
            .label(MapLabels::ApplyEvents)
            .label(MapLabels::TileOwner)
        );
        app.add_system(map_event_owner_cit
            .run_in_state(AppGlobalState::InGame)
            .label(MapLabels::ApplyEvents)
            .label(MapLabels::CitUpdate)
        );
        app.add_system(map_event_digit
            .run_in_state(AppGlobalState::InGame)
            .label(MapLabels::ApplyEvents)
            .label("map_event_digit")
        );
        app.add_system(drop_digits
            .run_in_state(AppGlobalState::InGame)
            .after(MapLabels::TileOwner)
            .after("map_event_digit")
            .label(MapLabels::TileDigit)
        );
        app.add_system(compute_fog_of_war::<Hex>
            .run_in_state(AppGlobalState::InGame)
            .after(MapLabels::TileOwner)
            .label(MapLabels::TileVisible)
        );
        app.add_system(compute_fog_of_war::<Sq>
            .run_in_state(AppGlobalState::InGame)
            .after(MapLabels::TileOwner)
            .label(MapLabels::TileVisible)
        );
        app.add_system(map_event_mine
            .run_in_state(AppGlobalState::InGame)
            .label(MapLabels::ApplyEvents)
            .label("map_event_mine")
        );
        app.add_system(drop_mines
            .run_in_state(AppGlobalState::InGame)
            .after(MapLabels::TileOwner)
            .after("map_event_mine")
            .label(MapLabels::TileMine)
        );
        app.add_plugin(gfx2d::MapGfx2dPlugin);
        #[cfg(feature = "dev")]
        app.add_system(debug_mapevents.label(MapLabels::ApplyEvents));
    }
}

pub struct MaxViewBounds(pub f32);

#[derive(Component)]
struct MapCleanup;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapEventKind {
    Owner {
        plid: PlayerId,
    },
    Digit {
        digit: u8,
    },
    Mine {
        state: Option<MineDisplayState>,
    },
    Road {
        state: Option<ProdState>,
    },
    Tower {
        state: Option<ProdState>,
    },
    Fort {
        state: Option<ProdState>,
    },
    Explosion {
        kind: MineKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapEvent {
    /// coordinate to update
    pub c: Pos,
    /// which view in a multi-view setup is the event for
    pub plid: PlayerId,
    /// the event
    pub kind: MapEventKind,
}

fn debug_mapevents(
    mut er_map: EventReader<MapEvent>,
) {
    for ev in er_map.iter() {
        trace!("{:?}", ev);
    }
}

fn setup_map(
    mut commands: Commands,
    descriptor: Option<Res<MapDescriptor>>,
    data: Option<Res<MapDataInitAny>>,
    mut done: Local<bool>,
) -> Progress {
    let descriptor = if let Some(descriptor) = descriptor {
        // reset for new game
        if descriptor.is_changed() {
            *done = false;
        }

        descriptor
    } else {
        return false.into();
    };

    if *done {
        return true.into();
    }

    if let Some(data) = data {
        match data.map.topology() {
            Topology::Hex => setup_map_topology::<Hex>(&mut commands, &*data),
            Topology::Sq => setup_map_topology::<Sq>(&mut commands, &*data),
            _ => unimplemented!()
        }

        *done = true;
        debug!("Setup tile entities for map: {:?}", descriptor);
    }

    true.into()
}

struct TileEntityIndex(MapAny<Entity>);

struct MineIndex(HashMap<Pos, Entity>);

struct CitIndex {
    by_pos: HashMap<Pos, Entity>,
    by_id: Vec<Entity>,
}

/// Per-tile component: the minesweeper digit
#[derive(Debug, Clone, Copy, Component)]
struct TileDigit(u8);
/// Per-tile component: the PlayerId of the owner
#[derive(Debug, Clone, Copy, Component)]
struct TileOwner(PlayerId);
/// Per-tile component: visibility (fog of war) state
#[derive(Debug, Clone, Copy, Component)]
struct TileFoW(bool);
/// Per-tile component: mine state
#[derive(Debug, Clone, Copy, Component)]
struct TileMine(Option<MineDisplayState>);
/// Per-tile component: is there a cit here?
#[derive(Debug, Clone, Copy, Component)]
struct TileCit(Entity);

/// Marker for Map Tiles
#[derive(Debug, Clone, Copy, Component)]
struct PlayableTileEntity;
/// Marker for Cits
#[derive(Debug, Clone, Copy, Component)]
struct CitEntity(CitId);
/// Marker for Towers
#[derive(Debug, Clone, Copy, Component)]
struct TowerEntity;
/// Marker for Forts
#[derive(Debug, Clone, Copy, Component)]
struct FortEntity;

/// How to render a mine?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MineDisplayState {
    /// Known to be sitting on its given tile
    Normal(MineKind),
    /// Supposed to be placed at its given tile, but unconfirmed by host
    Pending(MineKind),
    /// Known to be activated mine
    Active,
}

#[derive(Bundle)]
struct NonPlayableTileBundle {
    kind: TileKind,
    coord: TilePos,
}

#[derive(Bundle)]
struct PlayableTileBundle {
    marker: PlayableTileEntity,
    kind: TileKind,
    coord: TilePos,
    owner: TileOwner,
    vis: TileFoW,
}

#[derive(Bundle)]
struct LandTileExtrasBundle {
    digit: TileDigit,
    mine: TileMine,
}

#[derive(Bundle)]
struct CitBundle {
    cit: CitEntity,
    coord: TilePos,
    owner: TileOwner,
}

fn setup_map_topology<C: CompactMapCoordExt>(
    commands: &mut Commands,
    data: &MapDataInitAny,
) {
    let map: &MapData<C, _> = data.map.try_get().unwrap();

    let mut cit_index = CitIndex {
        by_id: Default::default(),
        by_pos: Default::default(),
    };

    for (i, pos) in data.cits.iter().enumerate() {
        let cit_e = commands.spawn_bundle(CitBundle {
            cit: CitEntity(i as u8),
            coord: TilePos::from(*pos),
            owner: TileOwner(PlayerId::Spectator),
        }).insert(MapCleanup).id();
        cit_index.by_id.push(cit_e);
        cit_index.by_pos.insert(*pos, cit_e);
    }

    let mut tile_index = MapData::new(map.size(), Entity::from_raw(0));

    // commands.insert_resource(MaxViewBounds(C::TILE_OFFSET.x.min(C::TILE_OFFSET.y) * map.size() as f32));
    for (c, init) in map.iter() {
        let tile_e = if init.kind.ownable() {
            let mut builder = commands.spawn_bundle(PlayableTileBundle {
                marker: PlayableTileEntity,
                kind: init.kind,
                coord: TilePos::from(c.as_pos()),
                owner: TileOwner(PlayerId::Spectator),
                vis: TileFoW(true),
            });
            if init.kind.is_land() {
                builder.insert_bundle(LandTileExtrasBundle {
                    digit: TileDigit(0),
                    mine: TileMine(None),
                });
            }
            if init.cit {
                builder.insert(TileCit(*cit_index.by_pos.get(&c.into()).unwrap()));
            }
            builder.insert(MapCleanup);
            builder.id()
        } else {
            commands.spawn_bundle(NonPlayableTileBundle {
                kind: init.kind,
                coord: TilePos::from(c.as_pos()),
            })
                .insert(MapCleanup).id()
        };

        tile_index[c] = tile_e;
    }

    let tile_index = TileEntityIndex(MapAny::from(tile_index));
    commands.insert_resource(tile_index);
    commands.insert_resource(MineIndex(Default::default()));
    commands.insert_resource(cit_index);
}

fn map_event_owner(
    mut evr_map: EventReader<MapEvent>,
    my_plid: Res<ActivePlid>,
    index: Res<TileEntityIndex>,
    mut q_tile: Query<&mut TileOwner, With<PlayableTileEntity>>,
) {
    for ev in evr_map.iter() {
        if ev.plid != my_plid.0 {
            continue;
        }
        if let MapEventKind::Owner { plid } = ev.kind {
            let e_tile = index.0[ev.c];
            if let Ok(mut tile_owner) = q_tile.get_mut(e_tile) {
                // do not try to avoid change detection!
                tile_owner.0 = plid;
            }
        }
    }
}

fn map_event_owner_cit(
    mut evr_map: EventReader<MapEvent>,
    my_plid: Res<ActivePlid>,
    index: Res<CitIndex>,
    mut q_cit: Query<&mut TileOwner, With<CitEntity>>,
) {
    for ev in evr_map.iter() {
        if ev.plid != my_plid.0 {
            continue;
        }
        if let MapEventKind::Owner { plid } = ev.kind {
            if let Some(e_cit) = index.by_pos.get(&ev.c) {
                if let Ok(mut owner) = q_cit.get_mut(*e_cit) {
                    // do not try to avoid change detection!
                    owner.0 = plid;
                }
            }
        }
    }
}

fn map_event_digit(
    mut evr_map: EventReader<MapEvent>,
    my_plid: Res<ActivePlid>,
    index: Res<TileEntityIndex>,
    mut q_tile: Query<&mut TileDigit, With<PlayableTileEntity>>,
) {
    for ev in evr_map.iter() {
        if ev.plid != my_plid.0 {
            continue;
        }
        if let MapEventKind::Digit { digit } = ev.kind {
            let e_tile = index.0[ev.c];
            if let Ok(mut tile_digit) = q_tile.get_mut(e_tile) {
                // do not try to avoid change detection!
                tile_digit.0 = digit;
            }
        }
    }
}

fn map_event_mine(
    mut evr_map: EventReader<MapEvent>,
    my_plid: Res<ActivePlid>,
    index: Res<TileEntityIndex>,
    mut q_tile: Query<&mut TileMine, With<PlayableTileEntity>>,
    mut mines: ResMut<MineIndex>,
) {
    for ev in evr_map.iter() {
        if ev.plid != my_plid.0 {
            continue;
        }
        if let MapEventKind::Mine { state } = ev.kind {
            let e_tile = index.0[ev.c];
            if let Ok(mut tile_mine) = q_tile.get_mut(e_tile) {
                // do not try to avoid change detection!
                tile_mine.0 = state;

                // maintain mine index
                if state.is_some() {
                    mines.0.insert(ev.c, e_tile);
                } else {
                    mines.0.remove(&ev.c);
                }
            }
        }
    }
}

fn compute_fog_of_war<C: Coord>(
    game_params: Option<Res<GameParams>>,
    my_plid: Res<ActivePlid>,
    index: Res<TileEntityIndex>,
    // FIXME PERF: this should be Mutated
    q_changed: Query<&TilePos, (With<PlayableTileEntity>, Changed<TileOwner>)>,
    q_owner: Query<&TileOwner, With<PlayableTileEntity>>,
    mut q_vis: Query<&mut TileFoW, With<PlayableTileEntity>>,
    mut dirty: Local<Vec<C>>,
) {
    if index.0.topology() != C::TOPOLOGY {
        return;
    }

    let radius = match game_params {
        Some(params) => params.radius_vis,
        None => 0,
    };

    if radius == 0 {
        return;
    }

    mw_common::game::map::compute_fog_of_war(
        radius,
        &mut *dirty,
        my_plid.0,
        q_changed.iter().map(|tp| Pos::from(*tp).into()),
        |c| {
            if c.ring() >= index.0.size() {
                return None;
            }
            let c_e = index.0[c.into()];
            q_owner.get(c_e).ok().map(|x| x.0)
        },
        |c, vis| {
            if c.ring() >= index.0.size() {
                return;
            }
            let c_e = index.0[c.into()];
            if let Ok(mut c_vis) = q_vis.get_mut(c_e) {
                if c_vis.0 != vis {
                    c_vis.0 = vis;
                }
            }
        },
    );
}

fn drop_digits(
    my_plid: Res<ActivePlid>,
    mut q_tile: Query<
        (&mut TileDigit, &TileOwner),
        (With<PlayableTileEntity>, Or<(Changed<TileDigit>, Changed<TileOwner>)>),
    >,
) {
    for (mut digit, owner) in q_tile.iter_mut() {
        if owner.0 != my_plid.0 {
            digit.0 = 0;
        }
    }
}

fn drop_mines(
    my_plid: Res<ActivePlid>,
    mut q_tile: Query<
        (&mut TileMine, &TileOwner),
        (With<PlayableTileEntity>, Or<(Changed<TileMine>, Changed<TileOwner>)>),
    >,
) {
    for (mut mine, owner) in q_tile.iter_mut() {
        if owner.0 != my_plid.0 {
            if let Some(display) = mine.0 {
                match display {
                    MineDisplayState::Active => (),
                    MineDisplayState::Normal(_) |
                    MineDisplayState::Pending(_) => {
                        mine.0 = None;
                    }
                }
            }
        }
    }
}
