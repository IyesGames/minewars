use mw_app_core::{driver::DriverGovernor, map::*, map::tile::*, map::cit::*};
use mw_common::{game::*, grid::*, plid::PlayerId};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnTransition { from: AppState::InGame, to: AppState::GameLoading },
        cleanup_mapgov,
    );
    app.add_systems(Update, (
        gen_simple_map
            .track_progress()
            .run_if(any_filter::<(With<SimpleMapGenerator>, With<DriverGovernor>)>),
        (
            setup_tile_entities
                .track_progress(),
            setup_cit_entities
                .track_progress(),
        )
            .run_if(any_filter::<With<MapGovernor>>),
    )
        .in_set(InStateSet(AppState::GameLoading)),
    );
}

/// Add this onto the Driver Governor to generate a simple
/// flat map for a local/offline session.
///
/// During the GameLoading state, this will
/// enable a system that sets up the Map Governor.
#[derive(Component)]
pub struct SimpleMapGenerator {
    pub topology: Topology,
    pub size: u8,
}

fn gen_simple_map(
    mut commands: Commands,
    q_map: Query<(), With<MapGovernor>>,
    q_driver: Query<&SimpleMapGenerator, With<DriverGovernor>>,
) -> Progress {
    if !q_map.is_empty() {
        return true.into();
    }
    let gen = q_driver.single();
    let mut empty_tile = MapTileDataOrig::default();
    empty_tile.set_kind(TileKind::Regular);
    empty_tile.set_item(ItemKind::Safe);
    empty_tile.set_region(0);
    let map_src = MapDataPos::new(gen.size, empty_tile);

    commands.spawn((
        MapGovernorBundle {
            cleanup: GameFullCleanup,
            marker: MapGovernor,
            desc: MapDescriptor {
                size: gen.size,
                topology: gen.topology,
            },
            map_src: MapDataOrig {
                map: map_src,
                cits: vec![],
            },
            grid_cursor: default(),
            grid_cursor_tile_entity: default(),
        },
    ));

    // for the sake of the progress bar not appearing like it is
    // going backwards (there will be new systems on the next frame
    // after we have spawned the Map Governor),
    // return a fake large total amount
    Progress {
        done: 1,
        total: 8,
    }
}

fn setup_tile_entities(
    mut commands: Commands,
    spreader: Res<WorkSpreader>,
    q_map: Query<(Entity, &MapDataOrig, Has<MapTileIndex>), With<MapGovernor>>,
) -> Progress {
    let (e_map, map_src) = match q_map.get_single() {
        Err(_) => return false.into(),
        Ok((_, _, true)) => return true.into(),
        Ok((e, orig, false)) => (e, orig),
    };
    if spreader.acquire() {
        return false.into();
    }

    let mut tile_index = MapTileIndex(
        MapDataPos::new(map_src.map.size(), Entity::PLACEHOLDER)
    );

    for (c, d) in map_src.map.iter() {
        let b_base = MapTileBundle {
            cleanup: GamePartialCleanup,
            marker: MwMapTile,
            kind: d.kind(),
            pos: MwTilePos(c.into()),
        };
        let e_tile = if d.kind().ownable() {
            let b_playable = PlayableTileBundle {
                tile: b_base,
                region: TileRegion(d.region()),
                owner: TileOwner(PlayerId::Neutral),
                vis: TileVisLevel::Visible,
            };
            if d.kind().is_land() {
                commands.spawn(LandTileBundle {
                    tile: b_playable,
                    digit_external: TileDigitExternal(MwDigit { digit: 0, asterisk: false }),
                    digit_internal: TileDigitInternal(MwDigit { digit: 0, asterisk: false }),
                    gent: TileGent::Empty,
                    roads: TileRoads(0),
                }).id()
            } else if d.kind().is_rescluster() {
                commands.spawn(ResClusterTileBundle {
                    tile: b_playable,
                }).id()
            } else {
                commands.spawn(b_playable).id()
            }
        } else {
            commands.spawn(b_base).id()
        };
        tile_index.0[c.into()] = e_tile;
    }

    commands.entity(e_map)
        .insert(tile_index);

    false.into()
}

fn setup_cit_entities(
    mut commands: Commands,
    spreader: Res<WorkSpreader>,
    q_map: Query<(Entity, &MapDataOrig, &MapTileIndex, Has<CitIndex>), With<MapGovernor>>,
) -> Progress {
    let (e_map, map_src, tile_index) = match q_map.get_single() {
        Err(_) => return false.into(),
        Ok((_, _, _, true)) => return true.into(),
        Ok((e, orig, tile_index, false)) => (e, orig, tile_index),
    };
    if spreader.acquire() {
        return false.into();
    }

    let mut cit_index = CitIndex {
        by_id: Vec::with_capacity(map_src.cits.len()),
        by_pos: HashMap::with_capacity(map_src.cits.len()),
    };

    for (i, cit_pos) in map_src.cits.iter().enumerate() {
        let cit_pos = *cit_pos;
        let e_cit = commands.spawn(
            CitBundle {
                cleanup: GamePartialCleanup,
                region: CitRegion(i as u8),
                owner: CitOwner(PlayerId::Neutral),
                economy: CitEconomy {
                    money: 0,
                    income: 0,
                    res: 0,
                    export: 255,
                    import: 255,
                },
            },
        ).id();
        cit_index.by_id.push(e_cit);
        cit_index.by_pos.insert(cit_pos, e_cit);
        commands.entity(tile_index.0[(cit_pos).into()])
            .insert(TileGent::Cit(i as u8));
    }

    commands.entity(e_map)
        .insert(cit_index);

    false.into()
}

fn cleanup_mapgov(
    mut commands: Commands,
    q_map: Query<Entity, With<MapGovernor>>,
) {
    commands.entity(q_map.single())
        .remove::<MapTileIndex>()
        .remove::<CitIndex>();
}
