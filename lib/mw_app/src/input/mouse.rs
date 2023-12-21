use bevy::input::{mouse::MouseButtonInput, ButtonState};

use crate::prelude::*;
use super::*;

pub struct MouseInputPlugin;

impl Plugin for MouseInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            collect_actions_mousebtn
                .in_set(GameInputSet::Collect)
                .run_if(on_event::<MouseButtonInput>()),
        ));
    }
}

fn collect_actions_mousebtn(
    settings: Res<AllSettings>,
    mut analogs: ResMut<ActiveAnalogs>,
    mut evr_mousebtn: EventReader<MouseButtonInput>,
    mut evw_action: EventWriter<InputAction>,
) {
    for ev in evr_mousebtn.iter() {
        if let Some(action) = settings.input.mouse.map.get(&ev.button) {
            match ev.state {
                ButtonState::Pressed => {
                    action.activate(
                        AnalogSource::MouseMotion,
                        &mut evw_action, &mut analogs,
                    );
                }
                ButtonState::Released => {
                    action.deactivate(
                        AnalogSource::MouseMotion,
                        &mut analogs,
                    );
                }
            }
        }
    }
}
