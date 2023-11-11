use crate::{prelude::*, tool::*, camera::GridCursor};

mod gamepad;
mod kbd;
mod mouse;
mod touch;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentTool>();
        app.add_plugins((
            gamepad::GamepadInputPlugin,
            kbd::KbdInputPlugin,
            mouse::MouseInputPlugin,
            touch::TouchInputPlugin,
        ));
        app.add_systems(Startup, setup_input);
        app.add_systems(Update, (
            input_tool_event.before(ToolEventHandlerSet),
        ));
    }
}

/// If any entities with this component exist, gameplay input handling is suspended
#[derive(Component)]
pub struct InhibitGameInput;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug, Reflect)]
#[derive(Serialize, Deserialize)]
pub enum InputAction {
    OpenDevConsole,
    SwitchTool(Tool),
    CycleToolPrev,
    CycleToolNext,
    UseCurrentTool,
    UseTool(Tool),
    ConfirmCurrentTool,
    CancelCurrentTool,
    GridCursorMoveHex,
    GridCursorMoveSq,
    PanCamera,
    RotateCamera,
    ZoomCamera,
    MinimapEnlarge,
    MinimapShrink,
}

#[derive(Resource, Default)]
pub struct CurrentTool(pub Tool);

pub fn new_map_with_minewars_defaults() -> InputMap<InputAction> {
    let mut map = InputMap::default() ;
    kbd::add_minewars_defaults(&mut map);
    mouse::add_minewars_defaults(&mut map);
    gamepad::add_minewars_defaults(&mut map);
    map
}

fn setup_input(mut commands: Commands) {
    commands.spawn(InputManagerBundle {
        action_state: ActionState::default(),
        input_map: new_map_with_minewars_defaults(),
    });
}

fn input_tool_event(
    crs: Res<GridCursor>,
    mut current_tool: ResMut<CurrentTool>,
    q_action: Query<&ActionState<InputAction>>,
    mut evw_tool: EventWriter<ToolEvent>,
) {
    for action_state in &q_action {
        for just_pressed in action_state.get_just_pressed() {
            match just_pressed {
                InputAction::UseCurrentTool => {
                    evw_tool.send(ToolEvent {
                        tool: current_tool.0,
                        state: ToolState::Select(crs.0),
                    });
                }
                InputAction::UseTool(tool) => {
                    evw_tool.send(ToolEvent {
                        tool,
                        state: ToolState::Select(crs.0),
                    });
                }
                InputAction::SwitchTool(tool) => {
                    evw_tool.send(ToolEvent {
                        tool: current_tool.0,
                        state: ToolState::Cancel,
                    });
                    current_tool.0 = tool;
                }
                InputAction::CancelCurrentTool => {
                    evw_tool.send(ToolEvent {
                        tool: current_tool.0,
                        state: ToolState::Cancel,
                    });
                }
                InputAction::ConfirmCurrentTool => {
                    evw_tool.send(ToolEvent {
                        tool: current_tool.0,
                        state: ToolState::Confirm,
                    });
                }
                _ => {},
            }
        }
    }
}
