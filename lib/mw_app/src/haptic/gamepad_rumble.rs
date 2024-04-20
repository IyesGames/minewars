use bevy::input::gamepad::{GamepadRumbleIntensity, GamepadRumbleRequest};

use crate::prelude::*;

use super::*;

pub struct HapticGamepadPlugin;

impl Plugin for HapticGamepadPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            haptic_gamepad_rumble
                .in_set(SetStage::WantChanged(HapticEventSS)),
        ));
    }
}

fn haptic_gamepad_rumble(
    gamepads: Res<Gamepads>,
    mut evr_haptic: EventReader<HapticEvent>,
    mut evw_rumble: EventWriter<GamepadRumbleRequest>,
) {
    let mut rumble = |gamepad, secs, strong, weak| {
        evw_rumble.send(GamepadRumbleRequest::Add {
            duration: Duration::from_secs_f64(secs),
            intensity: GamepadRumbleIntensity {
                strong_motor: strong,
                weak_motor: weak,
            },
            gamepad,
        });
    };
    for ev in evr_haptic.read() {
        for gamepad in gamepads.iter() {
            match ev.kind {
                HapticEventKind::ExplosionMineDeath => {
                    rumble(gamepad, 1.5, 1.0, 0.0);
                    rumble(gamepad, 1.0, 1.0, 1.0);
                }
                HapticEventKind::ExplosionOurTerritory => {
                    rumble(gamepad, 0.25, 0.5, 0.0);
                    rumble(gamepad, 0.125, 0.25, 0.5);
                }
                HapticEventKind::ExplosionForeignTerritory => {
                    rumble(gamepad, 0.25, 0.25, 0.0);
                    rumble(gamepad, 0.125, 0.125, 0.25);
                }
                HapticEventKind::BackgroundTremor => {
                    rumble(gamepad, 0.125, 0.125, 0.0);
                    rumble(gamepad, 0.0625, 0.125, 0.125);
                }
                HapticEventKind::ExplosionMineKill => {
                    rumble(gamepad, 1.25, 0.5, 0.0);
                    rumble(gamepad, 1.0, 0.5, 0.5);
                }
                HapticEventKind::ExplosionSomeoneDied => {
                    rumble(gamepad, 1.25, 0.25, 0.0);
                    rumble(gamepad, 1.0, 0.25, 0.25);
                }
                _ => todo!()
            }
        }
    }
}
