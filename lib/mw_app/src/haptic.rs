//! Everything to do with shaking and vibration
//!
//! This module contains our abstractions/events to represent haptics
//! and logic for managing and emitting them based on various game events.
//!
//! These events can then be handled by various other systems to be
//! translated into real effects that can be experienced by the user:
//!  - `crate::gfx2d::camera::shake`: screen shake (2D)
//!  - `crate::gfx3d::camera::shake`: screen shake (3D)
//!  - `mobile::haptic_android`: vibrates the device (Android)
//!  - `self::gamepad_rumble`: vibrates a gamepad
//!  - `self::buttplug`: vibrates buttplugs

use mw_common::{game::{event::{BackgroundEv, GameEvent, MapEv, MwEv, PlayerEv}, ItemKind}, grid::{Coord, Hex, Pos, Sq, Topology}, plid::PlayerId};

use crate::{map::{MapTileIndex, MapTopologySet, MapUpdateSet, TileOwner}, prelude::*, view::PlidViewing};

mod gamepad_rumble;
#[cfg(feature = "buttplug")]
mod buttplug;

pub struct HapticPlugin;

impl Plugin for HapticPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<HapticEvent>();
        app.configure_stage_set(Update, HapticEventSS, on_event::<HapticEvent>());
        app.add_plugins((
            gamepad_rumble::HapticGamepadPlugin,
            #[cfg(feature = "buttplug")]
            buttplug::HapticButtplugPlugin,
        ));
        app.add_systems(Update, (
            emit_haptic_events::<Hex>
                .in_set(MapTopologySet(Topology::Hex)),
            emit_haptic_events::<Sq>
                .in_set(MapTopologySet(Topology::Sq)),
        )
            .in_set(SetStage::WantChanged(GameOutEventSS))
            .in_set(SetStage::Provide(HapticEventSS))
            .after(MapUpdateSet::TileOwner)
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct HapticEventSS;

/// Events to trigger different kinds of shake/haptic effects.
///
/// Different systems can handle these events to implement the
/// appropriate effects via camera shake, hardware vibrators, etc.
#[derive(Event, Debug)]
pub struct HapticEvent {
    /// Grid/map position, if known
    pub pos: Option<Pos>,
    /// What causes the haptic?
    pub kind: HapticEventKind,
}

/// The different things that can cause haptic feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HapticEventKind {
    /// Ambience / background explosions
    BackgroundTremor,

    /// Explosion on player territory
    ExplosionOurTerritory,
    /// Explosion outside of player territory
    ExplosionForeignTerritory,

    /// Strike on player's mine
    ExplosionTheyDestroyOurMine,
    /// Enemy stepped on our mine
    ExplosionMineKill,
    /// Player stepped on a mine
    ExplosionMineDeath,
    /// Enemy stepped on someone else's mine
    ExplosionSomeoneDied,

    /// Player structure destroyed
    StructureDestroyedOur,
    /// Player destroyed a structure
    StructureDestroyedTheir,
}

fn emit_haptic_events<C: Coord>(
    mut evr_game: EventReader<GameEvent>,
    mut evw_haptic: EventWriter<HapticEvent>,
    viewing: Res<PlidViewing>,
    index: Res<MapTileIndex<C>>,
    q_owner: Query<&TileOwner>,
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

    // pass 1: accumulate info from events into buffers
    // (though some events are obvious and we can emit haptic immediately)
    for ev in evr_game.read() {
        if ev.plid != viewing.0 {
            continue;
        }
        match ev.ev {
            MwEv::Background(BackgroundEv::Tremor) => {
                evw_haptic.send(HapticEvent {
                    pos: None,
                    kind: HapticEventKind::BackgroundTremor,
                });
            }
            MwEv::Player { plid, ev: PlayerEv::Exploded { pos } } => {
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
            MwEv::Map { pos, ev: MapEv::Explode } => {
                setbit(pos, BUF_EXPLOSION_BIT);
            }
            MwEv::Map { pos, ev: MapEv::RevealItem { kind } } => {
                match kind {
                    ItemKind::Mine | ItemKind::Safe => {}
                    ItemKind::Decoy | ItemKind::Trap => {
                        setbit(pos, BUF_INHIBIT_BIT);
                    }
                }
            }
            MwEv::Map { pos, ev: MapEv::StructureGone } => {
                setbit(pos, BUF_STRUCTURE_BIT);
            }
            _ => {}
        }
    }

    // pass 2: resolve buffers to figure out what haptics to emit
    for (pos, bits) in buf.drain() {
        if bits & BUF_INHIBIT_BIT != 0 {
            continue;
        }

        let e_tile = index.0[pos.into()];

        let plid_owner = q_owner
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

