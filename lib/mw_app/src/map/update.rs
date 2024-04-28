use mw_common::game::event::*;

use crate::prelude::*;
use crate::view::{PlidViewing, ViewSwitchSet};
use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        (
            event_kind::<Hex>.in_set(MapUpdateSet::TileKind),
            event_owner::<Hex>.in_set(MapUpdateSet::TileOwner),
            event_digit::<Hex>.in_set(MapUpdateSet::TileDigit),
            (
                (event_gents::<Hex>, event_explosion::<Hex>).chain(),
            ).in_set(MapUpdateSet::TileGent),
        ).in_set(MapTopologySet(Topology::Hex)),
        (
            event_kind::<Sq>.in_set(MapUpdateSet::TileKind),
            event_owner::<Sq>.in_set(MapUpdateSet::TileOwner),
            event_digit::<Sq>.in_set(MapUpdateSet::TileDigit),
            (
                (event_gents::<Sq>, event_explosion::<Sq>).chain(),
            ).in_set(MapUpdateSet::TileGent),
        ).in_set(MapTopologySet(Topology::Sq)),
    )
        .in_set(NeedsMapSet)
        .in_set(SetStage::WantChanged(GameOutEventSS))
        .after(ViewSwitchSet));
    app.add_systems(Update, (
        alert_timer,
    ).in_set(NeedsMapSet));
}

fn event_kind<C: Coord>(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<&mut TileKind>,
) {
    for ev in evr.read() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::TileKind { kind }} = ev.ev {
            if let Ok(mut tilekind) = q_tile.get_mut(index.0[pos.into()]) {
                *tilekind = kind;
            }
        }
    }
}

fn event_owner<C: Coord>(
    mut commands: Commands,
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    cits: Res<CitIndex>,
    mut q_tile: Query<&mut TileOwner>,
    mut q_cit: Query<&mut CitOwner>,
) {
    for ev in evr.read() {
        if ev.plid != viewing.0 {
            continue;
        }
        match ev.ev {
            MwEv::Map { pos, ev: MapEv::Digit { .. }} => {
                let e_tile = index.0[pos.into()];
                if let Ok(mut owner) = q_tile.get_mut(e_tile) {
                    owner.0 = viewing.0;
                }
            }
            MwEv::Map { pos, ev: MapEv::Owner { plid }} => {
                let e_tile = index.0[pos.into()];
                if let Ok(mut owner) = q_tile.get_mut(e_tile) {
                    if owner.0 == viewing.0 && plid != viewing.0 {
                        commands.entity(e_tile).insert(
                            TileAlert(Timer::new(Duration::from_millis(1000), TimerMode::Once))
                        );
                    }
                    owner.0 = plid;
                    if let Some(e_cit) = cits.by_pos.get(&pos) {
                        let mut citowner = q_cit.get_mut(*e_cit).unwrap();
                        citowner.0 = plid;
                    }
                }
            }
            _ => continue,
        }
    }
}

fn event_digit<C: Coord>(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<&mut TileDigit>,
) {
    for ev in evr.read() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::Digit { digit, asterisk }} = ev.ev {
            if let Ok(mut tiledigit) = q_tile.get_mut(index.0[pos.into()]) {
                tiledigit.0 = digit;
                tiledigit.1 = asterisk;
            }
        }
    }
}

fn event_gents<C: Coord>(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<&mut TileGent>,
) {
    for ev in evr.read() {
        if ev.plid != viewing.0 {
            continue;
        }
        let (pos, gent) = match ev.ev {
            | MwEv::Map { pos, ev: MapEv::Unflag }
            | MwEv::Map { pos, ev: MapEv::Flag { plid: PlayerId::Neutral }} => {
                (pos, TileGent::Empty)
            }
            MwEv::Map { pos, ev: MapEv::Flag { plid }} => {
                (pos, TileGent::Flag(plid))
            }
            | MwEv::Map { pos, ev: MapEv::PlaceItem { kind }}
            | MwEv::Map { pos, ev: MapEv::RevealItem { kind }} => {
                (pos, TileGent::Item(kind))
            }
            // TODO: structures
            _ => continue,
        };
        if let Ok(mut tilegent) = q_tile.get_mut(index.0[pos.into()]) {
            match *tilegent {
                // Cits are important, protect them against bad updates
                TileGent::Cit(_) => continue,
                _ => {
                    *tilegent = gent;
                }
            }
        }
    }
}

fn event_explosion<C: Coord>(
    mut commands: Commands,
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<(Entity, &MwTilePos, &mut TileGent)>,
) {
    for ev in evr.read() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::Explode } = ev.ev {
            if let Ok((e, tilepos, mut tilegent)) = q_tile.get_mut(index.0[pos.into()]) {
                let kind = if let TileGent::Item(ItemKind::Decoy) = *tilegent {
                    TileExplosionKind::Decoy
                } else {
                    TileExplosionKind::Normal
                };
                if let TileGent::Item(_) = *tilegent {
                    *tilegent = TileGent::Empty;
                }
                commands.spawn((
                    ExplosionBundle {
                        pos: MwTilePos(tilepos.0),
                        explosion: TileExplosion(e, kind),
                        view: VisibleInView(viewing.0),
                    },
                ));
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
