use mw_common::grid::{Hex, Sq};

use crate::{prelude::*, tool::*, camera::GridCursor};

mod gamepad;
mod kbd;
mod mouse;
mod touch;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
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
        ));
        app.add_plugins((
            gamepad::GamepadInputPlugin,
            kbd::KbdInputPlugin,
            mouse::MouseInputPlugin,
            touch::TouchInputPlugin,
        ));
        app.add_systems(OnEnter(AppState::InGame), clear_game_input);
        app.add_systems(Update, (
            input_tool_event.in_set(GameInputSet::Process).before(ToolEventHandlerSet),
        ));
    }
}

#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameInputSet {
    Collect,
    Process,
}

/// If any entities with this component exist, gameplay input handling is suspended
#[derive(Component)]
pub struct InhibitGameInput;

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
    #[cfg(feature = "dev")]
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

#[derive(Resource, Default)]
pub struct CurrentTool(pub Tool);

#[derive(PartialEq, Eq, Clone, Copy, Debug, Hash)]
enum AnalogSource {
    MouseMotion,
    MouseScroll,
    GamepadAxis(GamepadAxisType),
    GamepadAxisAny,
}

#[derive(Resource, Default)]
struct ActiveAnalogs(HashMap<AnalogSource, AnalogInput>);

impl InputAction {
    fn activate(
        self,
        analog_source: AnalogSource,
        evw: &mut EventWriter<InputAction>,
        analogs: &mut ResMut<ActiveAnalogs>,
    ) {
        match self {
            InputAction::Analog(analog_input) => {
                analogs.0.insert(analog_source, analog_input);
            }
            _ => {
                evw.send(self);
            }
        }
    }
    fn deactivate(
        self,
        analog_source: AnalogSource,
        analogs: &mut ResMut<ActiveAnalogs>,
    ) {
        match self {
            InputAction::Analog(_) => {
                analogs.0.remove(&analog_source);
            }
            _ => {}
        }
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
    for ev in evr_action.iter() {
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
