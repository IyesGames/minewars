use bevy::input::gamepad::GamepadEvent;
use bevy::input::keyboard::KeyboardInput;
use bevy::input::mouse::MouseMotion;
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
    app.init_resource::<DetectedInputDevices>();
    app.init_resource::<CurrentTool>();
    app.init_resource::<ActiveAnalogs>();
    app.configure_sets(Update, (
        GameInputSet::Detect,
        GameInputSet::Collect
            .in_set(InStateSet(AppState::InGame))
            .run_if(rc_accepting_game_input)
            .after(GameInputSet::Detect),
        GameInputSet::Process
            .in_set(InStateSet(AppState::InGame))
            .after(GameInputSet::Collect),
        GameInputSet::ProcessEvents
            .in_set(InStateSet(AppState::InGame))
            .in_set(GameInputSet::Process)
            .run_if(on_event::<InputAction>()),
        InputDeviceEnabledSet::Kbd
            .after(GameInputSet::Detect)
            .run_if(rc_input_device(InputDeviceEnabledSet::Kbd)),
        InputDeviceEnabledSet::Mouse
            .after(GameInputSet::Detect)
            .run_if(rc_input_device(InputDeviceEnabledSet::Mouse)),
        InputDeviceEnabledSet::Touch
            .after(GameInputSet::Detect)
            .run_if(rc_input_device(InputDeviceEnabledSet::Touch)),
        InputDeviceEnabledSet::Gamepad
            .after(GameInputSet::Detect)
            .run_if(rc_input_device(InputDeviceEnabledSet::Gamepad)),
    ));
    app.add_plugins((
        gamepad::plugin,
        kbd::plugin,
        mouse::plugin,
        touch::plugin,
    ));
    app.add_systems(OnEnter(AppState::InGame), clear_game_input);
    app.add_systems(Update, (
        detect_input_devices
            .run_if(rc_detect_input_devices)
            .in_set(GameInputSet::Detect),
        input_tool_event
            .in_set(GameInputSet::ProcessEvents)
            .before(ToolEventHandlerSet),
    ));
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameInputSet {
    Detect,
    Collect,
    Process,
    ProcessEvents,
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputDeviceEnabledSet {
    Kbd,
    Mouse,
    Touch,
    Gamepad,
}

#[derive(Resource, Default)]
pub struct DetectedInputDevices {
    pub kbd: bool,
    pub mouse: bool,
    pub touch: bool,
    pub gamepad: bool,
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

fn rc_input_device(device: InputDeviceEnabledSet) -> impl Fn(Res<DetectedInputDevices>) -> bool {
    match device {
        InputDeviceEnabledSet::Kbd => |d: Res<DetectedInputDevices>| d.kbd,
        InputDeviceEnabledSet::Mouse => |d: Res<DetectedInputDevices>| d.mouse,
        InputDeviceEnabledSet::Touch => |d: Res<DetectedInputDevices>| d.touch,
        InputDeviceEnabledSet::Gamepad => |d: Res<DetectedInputDevices>| d.gamepad,
    }
}

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

fn rc_detect_input_devices(
    evr_mouse: Res<Events<MouseMotion>>,
    evr_kbd: Res<Events<KeyboardInput>>,
    evr_touch: Res<Events<TouchInput>>,
    evr_gamepad: Res<Events<GamepadEvent>>,
) -> bool {
    !evr_mouse.is_empty() ||
    !evr_kbd.is_empty() ||
    !evr_touch.is_empty() ||
    !evr_gamepad.is_empty()
}

fn detect_input_devices(
    evr_mouse: Res<Events<MouseMotion>>,
    evr_kbd: Res<Events<KeyboardInput>>,
    evr_touch: Res<Events<TouchInput>>,
    evr_gamepad: Res<Events<GamepadEvent>>,
    gamepads: Res<Gamepads>,
    mut detected: ResMut<DetectedInputDevices>,
) {
    if !detected.mouse && !evr_mouse.is_empty() {
        detected.mouse = true;
    }
    if !detected.kbd && !evr_kbd.is_empty() {
        detected.kbd = true;
    }
    if !detected.touch && !evr_touch.is_empty() {
        detected.touch = true;
    }
    if !evr_gamepad.is_empty() {
        detected.gamepad = gamepads.iter().count() != 0;
    }
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
