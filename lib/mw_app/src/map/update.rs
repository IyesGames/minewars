use mw_common::game::event::*;

use crate::{prelude::*, GameEventSet};
use crate::view::{PlidViewing, ViewSwitchSet};
use super::*;

pub struct MapUpdatePlugin;

impl Plugin for MapUpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            (
                event_kind::<Hex>.in_set(MapUpdateSet::TileKind),
                event_owner::<Hex>.in_set(MapUpdateSet::TileOwner),
                event_digit::<Hex>.in_set(MapUpdateSet::TileDigit),
                event_flag::<Hex>.in_set(MapUpdateSet::TileFlag),
                (
                    (event_item::<Hex>, event_explosion::<Hex>).chain(),
                ).in_set(MapUpdateSet::TileGent),
            ).in_set(MapTopologySet(Topology::Hex)),
            (
                event_kind::<Sq>.in_set(MapUpdateSet::TileKind),
                event_owner::<Sq>.in_set(MapUpdateSet::TileOwner),
                event_digit::<Sq>.in_set(MapUpdateSet::TileDigit),
                event_flag::<Sq>.in_set(MapUpdateSet::TileFlag),
                (
                    (event_item::<Sq>, event_explosion::<Sq>).chain(),
                ).in_set(MapUpdateSet::TileGent),
            ).in_set(MapTopologySet(Topology::Sq)),
        ).in_set(NeedsMapSet).after(GameEventSet).after(ViewSwitchSet));
        app.add_systems(Update, (
            alert_timer,
        ).in_set(NeedsMapSet));
    }
}

fn event_kind<C: Coord>(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<&mut TileKind>,
) {
    for ev in evr.iter() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::Tile { kind }} = ev.ev {
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
    mut q_tile: Query<&mut TileOwner>,
) {
    for ev in evr.iter() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::Owner { plid }} = ev.ev {
            let e_tile = index.0[pos.into()];
            if let Ok(mut owner) = q_tile.get_mut(e_tile) {
                if owner.0 == viewing.0 && plid != viewing.0 {
                    commands.entity(e_tile).insert(
                        TileAlert(Timer::new(Duration::from_millis(1000), TimerMode::Once))
                    );
                }
                owner.0 = plid;
            }
        }
    }
}

fn event_digit<C: Coord>(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<&mut TileDigit>,
) {
    for ev in evr.iter() {
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

fn event_flag<C: Coord>(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<&mut TileFlag>,
) {
    for ev in evr.iter() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::Flag { plid }} = ev.ev {
            if let Ok(mut tileflag) = q_tile.get_mut(index.0[pos.into()]) {
                tileflag.0 = plid;
            }
        }
    }
}

fn event_item<C: Coord>(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    mut q_tile: Query<&mut TileGent>,
) {
    for ev in evr.iter() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::Item { kind }} = ev.ev {
            if let Ok(mut tilegent) = q_tile.get_mut(index.0[pos.into()]) {
                match *tilegent {
                    TileGent::Empty | TileGent::Item(_) => {
                        *tilegent = TileGent::Item(kind);
                    }
                    TileGent::Cit(_) | TileGent::Structure(_) => {}
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
    for ev in evr.iter() {
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
