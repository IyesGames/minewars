use mw_common::grid::Pos;

use crate::prelude::*;

pub struct ToolPlugin;

impl Plugin for ToolPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<Tool>();
        app.add_event::<ToolEvent>();
        app.configure_set(Update, ToolEventHandlerSet.run_if(on_event::<ToolEvent>()));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ToolEventHandlerSet;

#[derive(States, Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
#[derive(Serialize, Deserialize)]
pub enum Tool {
    #[default]
    Explore,
    Flag,
    Reveal,
    Strike,
    Harvest,
    DeployMine,
    DeployDecoy,
    DeployTrap,
    Smoke,
    RemoveStructure,
    BuildRoad,
    BuildBridge,
    BuildWall,
    BuildTower,
}

#[derive(Event)]
pub struct ToolEvent {
    pub tool: Tool,
    pub state: ToolState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToolState {
    /// The user is about to use the tool on a tile, but the action is unconfirmed yet.
    Pending(Pos),
    /// The user has used the tool on a tile
    ///
    /// For some tools (direct action), this will perform a game action.
    /// For other tools (select-then-act), this will place a mark, and the action will
    /// be peformed on `Commit`.
    Select(Pos),
    /// The user cancels any Pending tile(s).
    Cancel,
    /// For select-then-act tools, clear selections
    Clear,
    /// For select-then-act tools, confirm
    Confirm,
}
