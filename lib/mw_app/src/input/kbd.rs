use bevy::input::keyboard::KeyboardInput;

use crate::prelude::*;
use super::*;

#[cfg(feature = "gfx2d")]
mod gfx2d;

pub fn plugin(app: &mut App) {
    #[cfg(feature = "gfx2d")]
    app.add_plugins(gfx2d::plugin);
    app.add_systems(Update, (
        collect_actions_key
            .in_set(GameInputSet::Collect)
            .run_if(on_event::<KeyboardInput>()),
    ));
}

fn collect_actions_key(
    settings: Res<AllSettings>,
    mut analogs: ResMut<ActiveAnalogs>,
    mut evr_kbd: EventReader<KeyboardInput>,
    mut evw_action: EventWriter<InputAction>,
    // to ignore repeats
    kbd: Res<ButtonInput<KeyCode>>,
) {
    for ev in evr_kbd.read() {
        if let Some(action) = settings.input.keyboard.keymap.get(&ev.key_code) {
            if kbd.just_pressed(ev.key_code) {
                activate_input(
                    *action,
                    AnalogSource::MouseMotion,
                    &mut evw_action, &mut analogs,
                );
            }
            if kbd.just_released(ev.key_code) {
                deactivate_input(
                    *action,
                    AnalogSource::MouseMotion,
                    &mut analogs,
                );
            }
        }
    }
}
