use crate::prelude::*;
use leafwing_input_manager::axislike::VirtualAxis;

use super::*;

pub struct KbdInputPlugin;

impl Plugin for KbdInputPlugin {
    fn build(&self, app: &mut App) {
    }
}

pub fn add_minewars_defaults(map: &mut InputMap<InputAction>) {
    map.insert_modified(Modifier::Shift, KeyCode::Grave, InputAction::OpenDevConsole);
    map.insert_modified(Modifier::Shift, KeyCode::Tab, InputAction::CycleToolPrev);
    map.insert(KeyCode::Tab, InputAction::CycleToolNext);
    map.insert(KeyCode::Return, InputAction::UseCurrentTool);
    map.insert(KeyCode::Space, InputAction::UseCurrentTool);
    map.insert(VirtualDPad::arrow_keys(), InputAction::PanCamera);
    map.insert(KeyCode::Q, InputAction::SwitchTool(Tool::DeployMine));
    map.insert(KeyCode::W, InputAction::SwitchTool(Tool::DeployDecoy));
    map.insert(KeyCode::E, InputAction::SwitchTool(Tool::DeployTrap));
    map.insert(KeyCode::R, InputAction::SwitchTool(Tool::Smoke));
    map.insert(KeyCode::A, InputAction::SwitchTool(Tool::Explore));
    map.insert(KeyCode::S, InputAction::SwitchTool(Tool::Flag));
    map.insert(KeyCode::D, InputAction::SwitchTool(Tool::Reveal));
    map.insert(KeyCode::F, InputAction::SwitchTool(Tool::Strike));
    map.insert(KeyCode::G, InputAction::SwitchTool(Tool::Harvest));
    map.insert(KeyCode::Z, InputAction::SwitchTool(Tool::RemoveStructure));
    map.insert(KeyCode::X, InputAction::SwitchTool(Tool::BuildRoad));
    map.insert(KeyCode::C, InputAction::SwitchTool(Tool::BuildBridge));
    map.insert(KeyCode::V, InputAction::SwitchTool(Tool::BuildWall));
    map.insert(KeyCode::B, InputAction::SwitchTool(Tool::BuildTower));
    map.insert(VirtualAxis {
        negative: KeyCode::BracketRight.into(),
        positive: KeyCode::BracketLeft.into(),
    }, InputAction::RotateCamera);
    map.insert(VirtualAxis {
        negative: KeyCode::Minus.into(),
        positive: KeyCode::Plus.into(),
    }, InputAction::ZoomCamera);
}
