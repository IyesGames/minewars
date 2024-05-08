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
        // Ignore event if we don't have a view for that plid set up
        let Some(e_plid) = plids.0.get(plid.i()) else {
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
            MwEv::DigitCapture { pos, digit, asterisk } => {
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
                // explosions should be managed with entity visibility
            },
            _ => {}
        }
    }
}

