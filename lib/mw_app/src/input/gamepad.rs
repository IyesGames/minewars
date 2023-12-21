use bevy::input::gamepad::GamepadEvent;

use crate::prelude::*;

use super::*;

pub struct GamepadInputPlugin;

impl Plugin for GamepadInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            collect_actions_gamepad
                .in_set(GameInputSet::Collect)
                .run_if(on_event::<GamepadEvent>()),
        ));
    }
}

fn collect_actions_gamepad(
    settings: Res<AllSettings>,
    mut analogs: ResMut<ActiveAnalogs>,
    mut evr_gamepad: EventReader<GamepadEvent>,
    mut evw_action: EventWriter<InputAction>,
) {
    for ev in evr_gamepad.iter() {
        match ev {
            GamepadEvent::Button(btn) => {
                if let Some(action) = settings.input.gamepad.buttonmap.get(&btn.button_type) {
                    if btn.value != 0.0 {
                        action.activate(
                            AnalogSource::GamepadAxisAny,
                            &mut evw_action, &mut analogs
                        );
                    } else {
                        action.deactivate(
                            AnalogSource::GamepadAxisAny,
                            &mut analogs
                        );
                    }
                }
            }
            GamepadEvent::Axis(axis) => {
                if let Some(action) = settings.input.gamepad.axismap.get(&axis.axis_type) {
                    if axis.value != 0.0 {
                        action.activate(
                            AnalogSource::GamepadAxis(axis.axis_type),
                            &mut evw_action, &mut analogs
                        );
                    } else {
                        action.deactivate(
                            AnalogSource::GamepadAxis(axis.axis_type),
                            &mut analogs
                        );
                    }
                }
            }
            GamepadEvent::Connection(_) => {}
        }
    }
}
