use mw_common::grid::{Hex, Sq};

use crate::prelude::*;
use crate::tool::*;
use crate::camera::GridCursor;

mod gamepad;
mod kbd;
mod mouse;
mod touch;

pub fn plugin(app: &mut App) {
    app.add_event::<InputAction>();
    app.init_resource::<CurrentTool>();
    app.init_resource::<ActiveAnalogs>();
    app.configure_sets(Update, (
        GameInputSet::Collect
            .in_set(InStateSet(AppState::InGame))
            .run_if(rc_accepting_game_input),
        GameInputSet::Process
            .in_set(InStateSet(AppState::InGame))
            .after(GameInputSet::Collect),
        GameInputSet::ProcessEvents
            .in_set(GameInputSet::Process)
            .run_if(on_event::<InputAction>()),
    ));
    app.add_plugins((
        gamepad::plugin,
        kbd::plugin,
        mouse::plugin,
        touch::plugin,
    ));
    app.add_systems(OnEnter(AppState::InGame), clear_game_input);
    app.add_systems(Update, (
        input_tool_event.in_set(GameInputSet::ProcessEvents).before(ToolEventHandlerSet),
    ));
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameInputSet {
    Collect,
    Process,
    ProcessEvents,
}

#[derive(Event, PartialEq, Clone, Copy, Debug)]
#[derive(Serialize, Deserialize)]
pub enum InputAction {
    Analog(AnalogInput),
    OpenDevConsole,
    SwitchTool(Tool),
    CycleToolPrev,
    CycleToolNext,
    UseCurrentTool,
    UseTool(Tool),
    ConfirmCurrentTool,
    CancelCurrentTool,
    GridCursorMoveHex(Hex),
    GridCursorMoveSq(Sq),
    PanCamera(Vec2),
    RotateCamera(f32),
    ZoomCamera(f32),
    DevDebug,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
#[derive(Serialize, Deserialize)]
pub enum AnalogInput {
    GridCursorMove,
    PanCamera,
    RotateCamera,
    ZoomCamera,
}

/// If any entities with this component exist, gameplay input handling is suspended
#[derive(Component)]
pub struct InhibitGameInput;

#[derive(Resource, Default)]
pub struct CurrentTool(pub Tool);

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum AnalogSource {
    MouseMotion,
    MouseScroll,
    GamepadLeftStick(Gamepad),
    GamepadRightStick(Gamepad),
    GamepadAnyStick(Gamepad),
}

impl AnalogSource {
    fn is_gamepad(self) -> bool {
        match self {
            | AnalogSource::GamepadRightStick(_)
            | AnalogSource::GamepadLeftStick(_)
            | AnalogSource::GamepadAnyStick(_) => true,
            _ => false,
        }
    }
}

#[derive(Resource, Default)]
struct ActiveAnalogs(HashMap<AnalogInput, AnalogSource>);

fn activate_input(
    input: InputAction,
    analog_source: AnalogSource,
    evw: &mut EventWriter<InputAction>,
    analogs: &mut ResMut<ActiveAnalogs>,
) {
    match input {
        InputAction::Analog(analog_input) => {
            analogs.0.insert(analog_input, analog_source);
        }
        _ => {
            evw.send(input);
        }
    }
}
fn deactivate_input(
    input: InputAction,
    _analog_source: AnalogSource,
    analogs: &mut ResMut<ActiveAnalogs>,
) {
    match input {
        InputAction::Analog(analog_input) => {
            analogs.0.remove(&analog_input);
        }
        _ => {}
    }
}

fn rc_accepting_game_input(
    q_inhibit: Query<(), With<InhibitGameInput>>,
    q_ui_interaction: Query<&Interaction>,
) -> bool {
    if !q_inhibit.is_empty() {
        return false;
    }
    let any_interaction = q_ui_interaction.iter().any(|i| *i != Interaction::None);
    !any_interaction
}

fn clear_game_input(
    mut ev: ResMut<Events<InputAction>>,
    mut analogs: ResMut<ActiveAnalogs>,
) {
    ev.clear();
    analogs.0.clear();
}

fn input_tool_event(
    crs: Res<GridCursor>,
    mut current_tool: ResMut<CurrentTool>,
    mut evr_action: EventReader<InputAction>,
    mut evw_tool: EventWriter<ToolEvent>,
) {
    for ev in evr_action.read() {
        match ev {
            InputAction::UseCurrentTool => {
                evw_tool.send(ToolEvent {
                    tool: current_tool.0,
                    state: ToolState::Select(crs.0),
                });
                debug!("Use Tool: {:?}", current_tool.0);
            }
            InputAction::UseTool(tool) => {
                evw_tool.send(ToolEvent {
                    tool: *tool,
                    state: ToolState::Select(crs.0),
                });
                debug!("Use Tool: {:?}", *tool);
            }
            InputAction::SwitchTool(tool) => {
                evw_tool.send(ToolEvent {
                    tool: current_tool.0,
                    state: ToolState::Cancel,
                });
                current_tool.0 = *tool;
                debug!("Current Tool: {:?}", current_tool.0);
            }
            InputAction::CancelCurrentTool => {
                evw_tool.send(ToolEvent {
                    tool: current_tool.0,
                    state: ToolState::Cancel,
                });
                debug!("Cancel Tool: {:?}", current_tool.0);
            }
            InputAction::ConfirmCurrentTool => {
                evw_tool.send(ToolEvent {
                    tool: current_tool.0,
                    state: ToolState::Confirm,
                });
                debug!("Confirm Tool: {:?}", current_tool.0);
            }
            _ => {},
        }
    }
}
