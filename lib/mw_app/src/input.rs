use mw_common::grid::{Hex, Sq};

use crate::prelude::*;
use crate::tool::*;

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
