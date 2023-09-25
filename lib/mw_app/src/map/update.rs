use mw_common::game::event::*;

use crate::{prelude::*, GameEventSet};
use crate::view::PlidViewing;
use super::*;

pub struct MapUpdatePlugin;

impl Plugin for MapUpdatePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            event_owner.in_set(MapUpdateSet::TileOwner),
        ).in_set(NeedsMapSet).after(GameEventSet));
    }
}

fn event_owner(
    mut evr: EventReader<GameEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex>,
    mut q_tile: Query<&mut TileOwner>,
) {
    for ev in evr.iter() {
        if ev.plid != viewing.0 {
            continue;
        }
        if let MwEv::Map { pos, ev: MapEv::Owner { plid }} = ev.ev {
            if let Ok(mut owner) = q_tile.get_mut(index.get_pos(pos)) {
                owner.0 = plid;
            }
        }
    }
}
