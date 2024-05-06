use mw_common::game::event::*;

use crate::prelude::*;
use super::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        event_map,
    )
        .in_set(ViewUpdateSet)
        .in_set(NeedsMapSet)
        .in_set(SetStage::WantChanged(GameOutEventSS))
    );
}

/// Apply *all* incoming game events to their respective views
fn event_map(
    mut evr: EventReader<GameEvent>,
    plids: Res<PlayersIndex>,
    mut q_view: Query<&mut ViewMapData>,
) {
    for ev in evr.read() {
        let plid = ev.plid;
        if let MwEv::Map { pos, ev } = &ev.ev {
            // Ignore event if we don't have a view for that plid set up
            let Some(e_plid) = plids.0.get(plid.i()) else {
                continue;
            };
            let Ok(mut view) = q_view.get_mut(*e_plid) else {
                continue;
            };
            let tile = &mut view.0[(*pos).into()];
            match ev {
                MapEv::TileKind { kind } => {
                    tile.set_kind(*kind);
                },
                MapEv::Owner { plid } => {
                    tile.set_owner(u8::from(*plid));
                },
                MapEv::Digit { digit, asterisk } => {
                    tile.set_owner(u8::from(plid));
                    tile.set_digit(*digit);
                    tile.set_asterisk(*asterisk);
                },
                | MapEv::PlaceItem { kind }
                | MapEv::RevealItem { kind } => {
                    tile.set_item(*kind);
                },
                MapEv::Flag { plid } => {
                    tile.set_flag(u8::from(*plid));
                },
                MapEv::Unflag => {
                    tile.set_flag(0);
                },
                MapEv::Explode => {
                    // clear any item from the tile
                    tile.set_item(ItemKind::Safe);
                    // explosions should be managed with entity visibility
                },
                MapEv::Smoke { state } => {
                    // smokes should be managed with entity visibility
                },
                MapEv::StructureBuildNew { kind, pts } => todo!(),
                MapEv::StructureReveal { kind } => todo!(),
                MapEv::StructureHp { hp } => todo!(),
                MapEv::StructureProgress { current, rate } => todo!(),
                MapEv::StructureCancel => todo!(),
                MapEv::StructureGone => todo!(),
                MapEv::Debug(_) => {}
            }
        }
    }
}

