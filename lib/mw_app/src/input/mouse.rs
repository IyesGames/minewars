use bevy::input::{mouse::MouseButtonInput, ButtonState};

use crate::prelude::*;
use super::*;

#[cfg(feature = "gfx2d")]
mod gfx2d;

pub fn plugin(app: &mut App) {
    #[cfg(feature = "gfx2d")]
    app.add_plugins(gfx2d::plugin);
    app.add_systems(Update, (
        collect_actions_mousebtn
            .in_set(SetStage::Provide(GameInputSS::Events))
            .in_set(SetStage::Provide(GameInputSS::Analogs))
            .run_if(on_event::<MouseButtonInput>()),
    ));
}

fn collect_actions_mousebtn(
    settings: Res<AllSettings>,
    mut analogs: ResMut<ActiveAnalogs>,
    mut evr_mousebtn: EventReader<MouseButtonInput>,
    mut evw_action: EventWriter<InputAction>,
) {
    for ev in evr_mousebtn.read() {
        if let Some(action) = settings.input.mouse.map.get(&ev.button) {
            match ev.state {
                ButtonState::Pressed => {
                    activate_input(
                        *action,
                        AnalogSource::MouseMotion,
                        &mut evw_action, &mut analogs,
                    );
                }
                ButtonState::Released => {
                    deactivate_input(
                        *action,
                        AnalogSource::MouseMotion,
                        &mut analogs,
                    );
                }
            }
        }
    }
}
