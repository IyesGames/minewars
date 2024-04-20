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
    settings: Res<AllSettings>,
    mut evr_haptic: EventReader<HapticEvent>,
    mut evw_rumble: EventWriter<GamepadRumbleRequest>,
) {
    for ev in evr_haptic.read() {
        for gamepad in gamepads.iter() {
            if let Some(waves) = settings.input.gamepad.haptics.get(&ev.kind) {
                for wave in waves.iter() {
                    evw_rumble.send(GamepadRumbleRequest::Add {
                        duration: Duration::from_secs_f32(wave.0),
                        intensity: GamepadRumbleIntensity {
                            strong_motor: wave.1,
                            weak_motor: wave.2,
                        },
                        gamepad,
                    });
                }
            }
        }
    }
}
