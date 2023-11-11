use leafwing_input_manager::axislike::VirtualAxis;

use crate::prelude::*;

use super::*;

pub struct GamepadInputPlugin;

impl Plugin for GamepadInputPlugin {
    fn build(&self, app: &mut App) {
    }
}

pub fn add_minewars_defaults(map: &mut InputMap<InputAction>) {
    map.insert(GamepadButtonType::South, InputAction::UseCurrentTool);
    map.insert(GamepadButtonType::West, InputAction::ConfirmCurrentTool);
    map.insert(GamepadButtonType::East, InputAction::CancelCurrentTool);
    map.insert(GamepadButtonType::LeftTrigger, InputAction::CycleToolPrev);
    map.insert(GamepadButtonType::RightTrigger, InputAction::CycleToolNext);
    map.insert(DualAxis::left_stick(), InputAction::GridCursorMoveHex);
    map.insert(DualAxis::left_stick(), InputAction::GridCursorMoveSq);
    map.insert(DualAxis::right_stick(), InputAction::PanCamera);
    map.insert(VirtualAxis {
        // FIXME
        negative: SingleAxis::negative_only(GamepadAxisType::LeftZ, 0.125).into(),
        positive: SingleAxis::negative_only(GamepadAxisType::RightZ, 0.125).inverted().into(),
    }, InputAction::RotateCamera);
    map.insert(VirtualAxis {
        negative: GamepadButtonType::DPadDown.into(),
        positive: GamepadButtonType::DPadUp.into(),
    }, InputAction::ZoomCamera);
}
