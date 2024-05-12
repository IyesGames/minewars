use mw_app_core::{driver::{GameOutEventSS, NeedsGameplaySessionSet}, haptic::{HapticEvent, HapticEventKind, HapticEventSS}, map::{tile::{MwMapTile, TileOwner, TileUpdateSS}, MapGovernor, MapTileIndex, NeedsMapGovernorSet}, session::{PlidViewing, SessionGovernor}};
use mw_common::{game::{event::{GameEvent, MwEv, PlayerEv}, ItemKind}, grid::Pos, plid::PlayerId};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, emit_haptic_events
        .in_set(NeedsGameplaySessionSet)
        .in_set(NeedsMapGovernorSet)
        .in_set(SetStage::Want(TileUpdateSS))
        .in_set(SetStage::WantChanged(GameOutEventSS))
        .in_set(SetStage::Provide(HapticEventSS))
    );
}

fn emit_haptic_events(
    mut evr_game: EventReader<GameEvent>,
    mut evw_haptic: EventWriter<HapticEvent>,
    q_session: Query<&PlidViewing, With<SessionGovernor>>,
    q_map: Query<&MapTileIndex, With<MapGovernor>>,
    q_tile: Query<&TileOwner, With<MwMapTile>>,
    mut buf: Local<HashMap<Pos, u8>>,
) {
    const BUF_EXPLOSION_BIT: u8 = 0b00000001;
    const BUF_DEATH_BIT: u8     = 0b00000010;
    const BUF_STRUCTURE_BIT: u8 = 0b00000100;
    const BUF_INHIBIT_BIT: u8   = 0b00001000;

    buf.clear();

    let mut setbit = |pos: Pos, bit: u8| {
        if let Some(v) = buf.get_mut(&pos) {
            *v |= bit;
        } else {
            buf.insert(pos, bit);
        }
    };

    let viewing = q_session.single();

    // pass 1: accumulate info from events into buffers
    // (though some events are obvious and we can emit haptic immediately)
    for ev in evr_game.read() {
        if ev.plid != viewing.0 {
            continue;
        }
        match ev.ev {
            MwEv::Tremor => {
                evw_haptic.send(HapticEvent {
                    pos: None,
                    kind: HapticEventKind::BackgroundTremor,
                });
            }
            MwEv::Player { plid, subplid: _, ev: PlayerEv::Exploded { pos, killer: _ } } => {
                if plid == viewing.0 {
                    evw_haptic.send(HapticEvent {
                        pos: Some(pos),
                        kind: HapticEventKind::ExplosionMineDeath,
                    });
                    setbit(pos, BUF_INHIBIT_BIT);
                } else {
                    setbit(pos, BUF_DEATH_BIT);
                }
            }
            MwEv::Explode { pos } => {
                setbit(pos, BUF_EXPLOSION_BIT);
            }
            MwEv::RevealItem { pos, item } => {
                match item {
                    ItemKind::Mine | ItemKind::Safe => {}
                    ItemKind::Decoy | ItemKind::Trap => {
                        setbit(pos, BUF_INHIBIT_BIT);
                    }
                }
            }
            MwEv::StructureGone { pos } => {
                setbit(pos, BUF_STRUCTURE_BIT);
            }
            _ => {}
        }
    }

    let index = q_map.single();

    // pass 2: resolve buffers to figure out what haptics to emit
    for (pos, bits) in buf.drain() {
        if bits & BUF_INHIBIT_BIT != 0 {
            continue;
        }

        let e_tile = index.0[pos.into()];

        let plid_owner = q_tile
            .get(e_tile)
            .map(|owner| owner.0)
            .unwrap_or(PlayerId::Neutral);

        let owned = plid_owner == viewing.0;
        let explosion = bits & BUF_EXPLOSION_BIT != 0;
        let death = bits & BUF_DEATH_BIT != 0;
        let structure = bits & BUF_STRUCTURE_BIT != 0;

        let kind = match (explosion, death, owned, structure) {
            (_, true, false, _) => {
                HapticEventKind::ExplosionSomeoneDied
            }
            (_, true, true, _) => {
                HapticEventKind::ExplosionMineKill
            }
            (true, false, false, false) => {
                HapticEventKind::ExplosionForeignTerritory
            }
            (true, false, true, false) => {
                HapticEventKind::ExplosionOurTerritory
            }
            (true, false, false, true) => {
                HapticEventKind::StructureDestroyedTheir
            }
            (true, false, true, true) => {
                HapticEventKind::StructureDestroyedOur
            }
            (false, false, _, _) => continue,
        };

        evw_haptic.send(HapticEvent {
            pos: Some(pos),
            kind,
        });
    }
}
