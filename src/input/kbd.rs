use bevy::input::keyboard::KeyboardInput;

use crate::prelude::*;
use super::*;

mod gfx2d;

pub struct KeyboardInputPlugin;

impl Plugin for KeyboardInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(gfx2d::Gfx2dKeyboardInputPlugin);
        app.add_systems(Update, (
            collect_actions_key
                .in_set(GameInputSet::Collect)
                .run_if(on_event::<KeyboardInput>()),
        ));
    }
}

fn collect_actions_key(
    settings: Res<AllSettings>,
    mut analogs: ResMut<ActiveAnalogs>,
    mut evr_kbd: EventReader<KeyboardInput>,
    mut evw_action: EventWriter<InputAction>,
    // to ignore repeats
    kbd: Res<Input<ScanCode>>,
) {
    for ev in evr_kbd.iter() {
        if let Some(action) = settings.input.keyboard.scanmap.get(&ScanCode(ev.scan_code))
            .or_else(|| ev.key_code.and_then(|k| settings.input.keyboard.keymap.get(&k)))
        {
            if kbd.just_pressed(ScanCode(ev.scan_code)) {
                activate_input(
                    *action,
                    AnalogSource::MouseMotion,
                    &mut evw_action, &mut analogs,
                );
            }
            if kbd.just_released(ScanCode(ev.scan_code)) {
                deactivate_input(
                    *action,
                    AnalogSource::MouseMotion,
                    &mut analogs,
                );
            }
        }
    }
}
