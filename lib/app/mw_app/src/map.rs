use mw_app_core::{driver::{GameOutEventSS, NeedsGameplaySessionSet}, map::{cit::*, tile::*, *}, session::{PlidViewing, SessionGovernor}, view::VisibleInView};
use mw_common::{game::*, grid::*, plid::PlayerId};

use crate::{prelude::*, settings::GameViewSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnTransition { exited: AppState::InGame, entered: AppState::GameLoading },
        cleanup_mapgov,
    );
    app.add_systems(Update,
        clear_tuq
            .in_set(NeedsMapGovernorSet)
            .in_set(SetStage::Prepare(TileUpdateSS))
    );
    app.add_systems(Update, (
        setup_tile_entities
            .track_progress(),
        setup_cit_entities
            .track_progress(),
    )
        .in_set(InStateSet(AppState::GameLoading))
        .in_set(NeedsMapGovernorSet),
    );
    app.add_systems(Update, (
        (
            update_tiles_from_gameevents_kind,
            update_tiles_from_gameevents_owner_digit
                .after(update_tiles_from_gameevents_kind),
            update_tiles_from_gameevents_gents
                .after(update_tiles_from_gameevents_kind),
            handle_gameevent_explosions
                .after(update_tiles_from_gameevents_gents),
        ),
    )
        .in_set(InStateSet(AppState::InGame))
        .in_set(SetStage::WantChanged(GameOutEventSS))
        .in_set(TileUpdateSet::External)
        .in_set(NeedsGameplaySessionSet)
        .in_set(NeedsMapGovernorSet)
    );
    app.add_systems(Update, (
        tile_alert
            .before(update_tiles_from_gameevents_owner_digit)
            .in_set(SetStage::WantChanged(GameOutEventSS))
            .in_set(NeedsGameplaySessionSet)
            .in_set(NeedsMapGovernorSet),
        alert_timer
            .run_if(any_with_component::<TileAlert>),
    )
        .in_set(InStateSet(AppState::InGame))
    );
}

fn clear_tuq(
    mut q_map: Query<&mut TileUpdateQueue, With<MapGovernor>>,
) {
    let mut tuq = q_map.single_mut();
    tuq.clear();
}

fn setup_tile_entities(
    mut commands: Commands,
    spreader: Res<WorkSpreader>,
    q_map: Query<(Entity, &MapDescriptor, &MapDataOrig, Has<MapTileIndex>), With<MapGovernor>>,
) -> Progress {
    let (e_map, desc, map_src) = match q_map.get_single() {
        Err(_) => return false.into(),
        Ok((_, _, _, true)) => return true.into(),
        Ok((e, desc, orig, false)) => (e, desc, orig),
    };
    if spreader.acquire() {
        return false.into();
    }

    let mut tile_index = MapTileIndex(
        MapDataPos::new(map_src.map.size(), Entity::PLACEHOLDER)
    );

    for (c, d) in map_src.map.iter() {
        match desc.topology {
            Topology::Hex => {
                let c = Hex::from(c);
                if c.ring() > desc.size {
                    continue;
                }
            },
            Topology::Sq => {
                let c = Sq::from(c);
                if c.ring() > desc.size {
                    continue;
                }
            },
        };
        let mut e_tile = commands.spawn(MapTileBundle {
            cleanup: GamePartialCleanup,
            marker: MwMapTile,
            kind: d.kind(),
            pos: MwTilePos(c),
        });
        if d.kind().ownable() {
            e_tile.insert(PlayableTileBundle {
                region: TileRegion(d.region()),
                owner: TileOwner(PlayerId::Neutral),
                vis: TileVisLevel::Visible,
            });
        }
        if d.kind().is_land() {
            e_tile.insert(LandTileBundle::default());
        }
        if d.kind().is_rescluster() {
            e_tile.insert(ResClusterTileBundle {
            });
        }
        tile_index.0[c.into()] = e_tile.id();
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
                marker: MwCit,
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

fn update_tiles_from_gameevents_kind(
    mut commands: Commands,
    mut evr: EventReader<GameEvent>,
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    mut q_map: Query<(&mut TileUpdateQueue, &MapTileIndex), With<MapGovernor>>,
    mut q_tile: Query<(Entity, &mut TileKind), With<MwMapTile>>,
) {
    let viewing = q_session.single();
    let (mut tuq, index) = q_map.single_mut();
    for ev in evr.read() {
        // Ignore if it is not our event
        if !ev.plids.contains(viewing.0) {
            continue;
        }
        match ev.ev {
            MwEv::TileKind { pos, kind } => {
                let Ok((e, mut tilekind)) = q_tile.get_mut(index.0[pos]) else {
                    continue;
                };
                if tilekind.is_rescluster() && !kind.is_rescluster() {
                    // destroying a rescluster
                    commands.entity(e).remove::<ResClusterTileBundle>();
                }
                if !tilekind.is_rescluster() && kind.is_rescluster() {
                    // creating a rescluster
                    commands.entity(e).insert(ResClusterTileBundle {
                    });
                }
                if tilekind.is_land() && !kind.is_land() {
                    // no longer land
                    commands.entity(e).remove::<LandTileBundle>();
                }
                if !tilekind.is_land() && kind.is_land() {
                    // is now land
                    commands.entity(e).insert(LandTileBundle::default());
                }
                if tilekind.ownable() && !kind.ownable() {
                    // no longer playable
                    commands.entity(e)
                        .remove::<LandTileBundle>()
                        .remove::<ResClusterTileBundle>()
                        .remove::<PlayableTileBundle>();
                }
                if !tilekind.ownable() && kind.ownable() {
                    error!("Tile at {:?} went from kind {:?} to kind {:?}. Don't know how to handle non-playable tiles becoming playable!", pos, tilekind, kind);
                }
                *tilekind = kind;
                tuq.queue_one(e);
            },
            _ => {}
        }
    }
}

fn update_tiles_from_gameevents_owner_digit(
    mut evr: EventReader<GameEvent>,
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    mut q_map: Query<(&mut TileUpdateQueue, &MapTileIndex), With<MapGovernor>>,
    mut q_tile: Query<(Entity, &mut TileOwner, &mut TileDigitGame), With<MwMapTile>>,
) {
    let viewing = q_session.single();
    let (mut tuq, index) = q_map.single_mut();
    for ev in evr.read() {
        // Ignore if it is not our event
        if !ev.plids.contains(viewing.0) {
            continue;
        }
        match ev.ev {
            MwEv::DigitCapture { pos, digit } => {
                let Ok((e, mut owner, mut dig)) = q_tile.get_mut(index.0[pos]) else {
                    continue;
                };
                owner.0 = viewing.0;
                dig.0 = digit;
                tuq.queue_one(e);
            },
            MwEv::TileOwner { pos, plid } => {
                let Ok((e, mut owner, mut dig)) = q_tile.get_mut(index.0[pos]) else {
                    continue;
                };
                owner.0 = plid;
                if plid != viewing.0 {
                    dig.0 = MwDigit::default();
                }
                tuq.queue_one(e);
            }
            _ => {}
        }
    }
}

fn update_tiles_from_gameevents_gents(
    mut evr: EventReader<GameEvent>,
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    mut q_map: Query<(&mut TileUpdateQueue, &MapTileIndex), With<MapGovernor>>,
    mut q_tile: Query<(Entity, &mut TileGent), With<MwMapTile>>,
) {
    let viewing = q_session.single();
    let (mut tuq, index) = q_map.single_mut();
    for ev in evr.read() {
        // Ignore if it is not our event
        if !ev.plids.contains(viewing.0) {
            continue;
        }
        let (pos, newgent) = match ev.ev {
            MwEv::Flag { pos, plid: PlayerId::Neutral } => {
                (pos, TileGent::Empty)
            }
            MwEv::Flag { pos, plid } => {
                (pos, TileGent::Flag(plid))
            }
            MwEv::RevealItem { pos, item } => {
                (pos, TileGent::Item(item))
            }
            // TODO: structures
            _ => continue,
        };
        let Ok((e, mut gent)) = q_tile.get_mut(index.0[pos]) else {
            continue;
        };
        // Cits are important, protect them against bad updates
        if let TileGent::Cit(_) = *gent {
            continue;
        }
        *gent = newgent;
        tuq.queue_one(e);
    }
}

fn handle_gameevent_explosions(
    mut commands: Commands,
    mut evr: EventReader<GameEvent>,
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    mut q_map: Query<(&mut TileUpdateQueue, &MapTileIndex), With<MapGovernor>>,
    mut q_tile: Query<(Entity, &mut TileGent), With<MwMapTile>>,
) {
    let viewing = q_session.single();
    let (mut tuq, index) = q_map.single_mut();
    for ev in evr.read() {
        // Ignore if it is not our event
        if !ev.plids.contains(viewing.0) {
            continue;
        }
        if let MwEv::Explode { pos } = ev.ev {
            let Ok((e, mut gent)) = q_tile.get_mut(index.0[pos]) else {
                continue;
            };
            let item = if let TileGent::Item(item) = *gent {
                Some(item)
            } else {
                None
            };
            if let TileGent::Item(_) = *gent {
                *gent = TileGent::Empty;
            }
            commands.spawn((
                ExplosionBundle {
                    pos: MwTilePos(pos),
                    explosion: TileExplosion {
                        e, item,
                    },
                    view: VisibleInView(viewing.0),
                },
            ));
            tuq.queue_one(e);
        }
    }
}

fn tile_alert(
    mut commands: Commands,
    settings: Settings,
    mut evr: EventReader<GameEvent>,
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    q_map: Query<&MapTileIndex, With<MapGovernor>>,
    q_tile: Query<(Entity, &TileOwner), With<MwMapTile>>,
) {
    let viewing = q_session.single();
    let index = q_map.single();
    let dur_ms = settings.get::<GameViewSettings>().unwrap().tile_alert_duration_ms;
    for ev in evr.read() {
        // Ignore if it is not our event
        if !ev.plids.contains(viewing.0) {
            continue;
        }
        if let MwEv::TileOwner { pos, plid } = ev.ev {
            let Ok((e, owner)) = q_tile.get(index.0[pos]) else {
                continue;
            };
            if owner.0 == viewing.0 && plid != viewing.0 {
                commands.entity(e).insert(
                    TileAlert(Timer::new(Duration::from_millis(dur_ms as u64), TimerMode::Once))
                );
            }
        }
    }
}

fn alert_timer(
    time: Res<Time>,
    mut commands: Commands,
    mut q_alert: Query<(Entity, &mut TileAlert)>,
) {
    for (e, mut alert) in &mut q_alert {
        alert.0.tick(time.delta());
        if alert.0.finished() {
            commands.entity(e).remove::<TileAlert>();
        }
    }
}
