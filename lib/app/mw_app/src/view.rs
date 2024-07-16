use bevy::ecs::system::SystemState;
use mw_app_core::{driver::GameOutEventSS, map::{tile::*, MapDescriptor, MapGovernor, MapTileIndex}, session::{PlayersIndex, PlidViewing, SessionGovernor}, view::*};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        view_update_from_gameevents,
    )
        .in_set(MultiViewEnabledSet)
        .in_set(SetStage::Provide(ViewSS::Update))
        .in_set(SetStage::WantChanged(GameOutEventSS))
    );
    app.add_systems(Update, (
        switch_view_despawn,
        switch_view_showhide,
        switch_view_update_map,
    )
        .in_set(MultiViewEnabledSet)
        .in_set(SetStage::Provide(ViewSS::Switch))
        .run_if(rc_viewswitch)
    );
}

fn rc_viewswitch(
    q_session: Query<Ref<PlidViewing>, With<SessionGovernor>>,
) -> bool {
    q_session.get_single()
        .map(|plid| plid.is_changed())
        .unwrap_or(false)
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
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    mut q: Query<(&mut Visibility, &VisibleInView)>,
) {
    let viewing = q_session.single();
    for (mut vis, viewvis) in &mut q {
        if viewvis.0 == viewing.0 {
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn switch_view_update_map(
    world: &mut World,
    ss: &mut SystemState<(
        Query<(&PlayersIndex, &PlidViewing), With<SessionGovernor>>,
        Query<(&mut TileUpdateQueue, &MapTileIndex, &MapDescriptor), With<MapGovernor>>,
        Query<&ViewMapData>,
    )>,
    mut temp_data: Local<Option<ViewMapData>>,
    mut temp_tileindex: Local<Option<MapDataPos<Entity>>>,
) {
    let topology;
    {
        let (q_session, mut q_map, q_plid) = ss.get_mut(world);
        let (players, viewing) = q_session.single();
        let (mut tuq, mapindex, mapdesc) = q_map.single_mut();
        tuq.queue_all();
        let Ok(viewdata) = q_plid.get(players.e_plid[viewing.0.i()]) else {
            error!("View for {:?} does not exist!", viewing.0);
            return;
        };
        topology = mapdesc.topology;
        *temp_data = Some(viewdata.clone());
        *temp_tileindex = Some(mapindex.0.clone());
    }
    let viewdata = temp_data.take().unwrap();
    for (c, e) in temp_tileindex.take().unwrap().iter() {
        let tiledata = viewdata.0[c];
        let mut emut = world.entity_mut(*e);
        emut.remove::<(MapTileBundle, PlayableTileBundle, LandTileBundle, ResClusterTileBundle)>();
        emut.insert(MapTileBundle {
            cleanup: GamePartialCleanup,
            marker: MwMapTile,
            kind: tiledata.kind(),
            pos: MwTilePos(c),
        });
        if tiledata.kind().ownable() {
            emut.insert(PlayableTileBundle {
                region: TileRegion(tiledata.region()),
                owner: TileOwner(PlayerId::from(tiledata.owner())),
                vis: TileVisLevel::Visible,
            });
        }
        if tiledata.kind().is_land() {
            emut.insert(LandTileBundle {
                digit_internal: TileDigitInternal::default(),
                digit_external: TileDigitExternal(MwDigit { digit: tiledata.digit(), asterisk: tiledata.asterisk() }),
                gent: if tiledata.has_structure() {
                    TileGent::Structure(tiledata.structure())
                } else if tiledata.flag() != PlayerId::Neutral {
                    TileGent::Flag(tiledata.flag().into())
                } else if tiledata.item() != ItemKind::Safe {
                    TileGent::Item(tiledata.item())
                } else {
                    TileGent::Empty
                },
                roads: TileRoads(if tiledata.structure() == StructureKind::Road {
                    match topology {
                        Topology::Hex => {
                            let c = Hex::from(c);
                            viewdata.0.get_ringmask(c, |d| d.structure() == StructureKind::Road)
                        }
                        Topology::Sq => {
                            let c = Sq::from(c);
                            viewdata.0.get_ringmask(c, |d| d.structure() == StructureKind::Road)
                        }
                    }
                } else {
                    0
                }),
            });
        }
        if tiledata.kind().is_rescluster() {
            emut.insert(ResClusterTileBundle {
            });
        }
    }
}

fn view_update_from_gameevents(
    mut evr: EventReader<GameEvent>,
    q_session: Query<&PlayersIndex, With<SessionGovernor>>,
    mut q_view: Query<&mut ViewMapData>,
) {
    let index = q_session.single();
    for ev in evr.read() {
        for plid in ev.plids.iter(None) {
            // Ignore event if we don't have a view for that plid set up
            let Some(e_plid) = index.e_plid.get(plid.i()) else {
                continue;
            };
            let Ok(mut view) = q_view.get_mut(*e_plid) else {
                continue;
            };
            match ev.ev {
                MwEv::TileKind { pos, kind } => {
                    let tile = &mut view.0[pos];
                    tile.set_kind(kind);
                },
                MwEv::TileOwner { pos, plid } => {
                    let tile = &mut view.0[pos];
                    tile.set_owner(u8::from(plid));
                },
                MwEv::DigitCapture { pos, digit: MwDigit { digit, asterisk } } => {
                    let tile = &mut view.0[pos];
                    tile.set_owner(u8::from(plid));
                    tile.set_digit(digit);
                    tile.set_asterisk(asterisk);
                },
                MwEv::RevealItem { pos, item } => {
                    let tile = &mut view.0[pos];
                    tile.set_item(item);
                },
                MwEv::Flag { pos, plid } => {
                    let tile = &mut view.0[pos];
                    tile.set_flag(u8::from(plid));
                },
                MwEv::Explode { pos } => {
                    let tile = &mut view.0[pos];
                    // clear any item from the tile
                    tile.set_item(ItemKind::Safe);
                },
                _ => {}
            }
        }
    }
}
