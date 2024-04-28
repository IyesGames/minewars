use bevy::input::gamepad::GamepadEvent;

use crate::prelude::*;
use super::*;

#[cfg(feature = "gfx2d")]
mod gfx2d;

pub fn plugin(app: &mut App) {
    #[cfg(feature = "gfx2d")]
    app.add_plugins(gfx2d::plugin);
    app.add_systems(Update, (
        collect_actions_gamepad
            .in_set(SetStage::Provide(GameInputSS::Events))
            .in_set(SetStage::Provide(GameInputSS::Analogs))
            .run_if(on_event::<GamepadEvent>()),
    ));
}

fn collect_actions_gamepad(
    settings: Res<AllSettings>,
    mut analogs: ResMut<ActiveAnalogs>,
    axes: Res<Axis<GamepadAxis>>, // for cross-checking X/Y
    mut evr_gamepad: EventReader<GamepadEvent>,
    mut evw_action: EventWriter<InputAction>,
) {
    for ev in evr_gamepad.read() {
        match ev {
            GamepadEvent::Button(btn) => {
                if let Some(action) = settings.input.gamepad.buttonmap.get(&btn.button_type) {
                    if btn.value != 0.0 {
                        activate_input(
                            *action,
                            AnalogSource::GamepadAnyStick(btn.gamepad),
                            &mut evw_action, &mut analogs
                        );
                    } else {
                        deactivate_input(
                            *action,
                            AnalogSource::GamepadAnyStick(btn.gamepad),
                            &mut analogs
                        );
                    }
                }
            }
            GamepadEvent::Axis(axis) => {
                if let Some(action) = settings.input.gamepad.axismap.get(&axis.axis_type) {
                    let analog_source = match axis.axis_type {
                        | GamepadAxisType::LeftStickX
                        | GamepadAxisType::LeftStickY => AnalogSource::GamepadLeftStick(axis.gamepad),
                        | GamepadAxisType::RightStickX
                        | GamepadAxisType::RightStickY => AnalogSource::GamepadRightStick(axis.gamepad),
                        _ => continue,
                    };
                    if get_joystick(analog_source, &axes) != Vec2::ZERO {
                        activate_input(
                            *action,
                            analog_source,
                            &mut evw_action, &mut analogs
                        );
                    } else {
                        deactivate_input(
                            *action,
                            analog_source,
                            &mut analogs
                        );
                    }
                }
            }
            GamepadEvent::Connection(_) => {}
        }
    }
}

fn get_joystick(
    source: AnalogSource,
    axes: &Axis<GamepadAxis>,
) -> Vec2 {
    match source {
        AnalogSource::GamepadLeftStick(gamepad) => {
            let x = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::LeftStickX,
            }).unwrap_or(0.0);
            let y = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::LeftStickY,
            }).unwrap_or(0.0);
            Vec2::new(x, y)
        }
        AnalogSource::GamepadRightStick(gamepad) => {
            let x = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::RightStickX,
            }).unwrap_or(0.0);
            let y = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::RightStickY,
            }).unwrap_or(0.0);
            Vec2::new(x, y)
        }
        AnalogSource::GamepadAnyStick(gamepad) => {
            let xl = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::LeftStickX,
            }).unwrap_or(0.0);
            let yl = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::LeftStickY,
            }).unwrap_or(0.0);
            let xr = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::RightStickX,
            }).unwrap_or(0.0);
            let yr = axes.get(GamepadAxis {
                gamepad,
                axis_type: GamepadAxisType::RightStickY,
            }).unwrap_or(0.0);
            let l = Vec2::new(xl, yl);
            let r = Vec2::new(xr, yr);
            if l.length_squared() > r.length_squared() {
                l
            } else {
                r
            }
        }
        _ => Vec2::ZERO
    }
}
