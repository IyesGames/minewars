use bevy::ecs::{schedule::ScheduleLabel, system::SystemId};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<InputActionName>();
    app.register_type::<InputAnalogName>();
}

#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputActionOnPress(pub InputActionName);
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputActionOnRelease(pub InputActionName);
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputAnalogOnStart(pub InputAnalogName);
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct InputAnalogOnStop(pub InputAnalogName);

/// Stage Sets related to input handling
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameInputSS {
    Setup, // in Startup
    Detect, // in Update
    Handle, // in Update
}

/// System Sets for each class of input device.
///
/// Adds run conditions based on `DetectedInputDevices`.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum InputDeviceSet {
    Keyboard,
    Mouse,
    Touch,
    Gamepad,
}

/// System Set to run on Bevy keyboard input events.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OnKeyboardEventSet;

/// System Set to run on Bevy mouse scroll events.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OnMouseScrollEventSet;

/// System Set to run on Bevy mouse button events.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OnMouseButtonEventSet;

/// System Set to run on Bevy mouse motion or cursor moved events.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OnMouseMotionEventSet;

/// System Set for anything that can be considered "gameplay input"
///
/// Allows easily disabling all of that, using a RC.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GameInputSet;

#[derive(Bundle, Default)]
pub struct InputGovernorCoreBundle {
    pub marker: InputGovernor,
    pub detected: DetectedInputDevices,
    pub toolbox: Toolbox,
}

/// Marker for the Input Governor
#[derive(Component, Default)]
pub struct InputGovernor;

/// What input devices has the user been found to be using?
///
/// As soon as we notice input events from a particular kind
/// of device, we note that here. This allows us to then enable
/// functionality and UI specific to that kind of input device.
#[derive(Component, Default)]
pub struct DetectedInputDevices {
    pub kbd: bool,
    pub mouse: bool,
    pub touch: bool,
    pub gamepad: bool,
}

/// Gameplay Input Kill Switch
///
/// Add this component to the Input Governor to temporarily
/// disable all gameplay-related input.
#[derive(Component, Default)]
pub struct InhibitGameInput;

/// The set of tools to be used for the current game mode.
#[derive(Component, Default)]
pub struct Toolbox {
    /// All tools the user is allowed to activate,
    /// in some sort of logical order.
    pub tools: Vec<Entity>,
}

/// Bundle for entities representing Tools.
///
/// Tools are modal input actions that use the grid cursor.
#[derive(Bundle)]
pub struct ToolBundle {
    pub marker: Tool,
    pub callback: ToolCallback,
}

/// Marker for Tools
#[derive(Component)]
pub struct Tool;

/// Is the tool allowed to be used?
#[derive(Component)]
pub struct ToolEnabled;

/// Is the tool currently selected for use?
#[derive(Component)]
pub struct ToolActive;

/// Systems to run for a given tool
#[derive(Component)]
pub struct ToolCallback {
    /// Called when the user activates the tool
    pub on_activate: Option<SystemId>,
    /// Called when the user deactivates the tool
    pub on_deactivate: Option<SystemId>,
    /// Called when the user wants to use the tool at the grid cursor
    pub on_use: Option<SystemId>,
}

/// Bundle for entities representing Input Actions
#[derive(Bundle)]
pub struct InputActionBundle {
    pub marker: InputAction,
    pub name: InputActionName,
}

/// Marker for input actions
#[derive(Component)]
pub struct InputAction;

/// Is the action allowed to be used?
#[derive(Component)]
pub struct InputActionEnabled;

/// Is the action currently held down?
#[derive(Component)]
pub struct InputActionActive;

/// Internal ID for the input action.
///
/// Used to enable use cases like storing settings.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputActionName(pub String);

/// Bundle for entities representing Analog Controls
///
/// When an analog action is being performed, an
/// `InputAnalogActive` and an `AnalogSource*` component
/// will be inserted, indicating the source of the analog values
/// (which input device is performing the action).
#[derive(Bundle)]
pub struct InputAnalogBundle {
    pub marker: InputAnalog,
    pub name: InputAnalogName,
}

/// Marker for Analog Inputs
#[derive(Component)]
pub struct InputAnalog;

/// Is the Analog Input allowed to be used?
#[derive(Component)]
pub struct InputAnalogEnabled;

/// Is the analog currently in operation?
///
/// There should also be an `AnalogSource*` component
/// to indicate what device is operating it.
#[derive(Component)]
pub struct InputAnalogActive;

/// Internal ID for the analog input.
///
/// Used to enable use cases like storing settings.
#[derive(Component, Reflect, Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputAnalogName(pub String);

/// Action currently performed by mouse motion or pointer/cursor position.
/// Which is more appropriate is an implementation detail.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnalogSourceMouseMotion;

/// Action currently performed by mouse scrolling / wheel
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnalogSourceMouseScroll;

/// Action currently performed by gamepad joystick
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnalogSourceGamepadStick {
    pub gamepad: Gamepad,
    pub left: bool,
    pub right: bool,
}

/// Action currently performed by gamepad Z axis / trigger
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash)]
pub struct AnalogSourceGamepadZ {
    pub gamepad: Gamepad,
    pub left: bool,
    pub right: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AnalogSource {
    MouseMotion(AnalogSourceMouseMotion),
    MouseScroll(AnalogSourceMouseScroll),
    GamepadStick(AnalogSourceGamepadStick),
    GamepadZ(AnalogSourceGamepadZ),
}

/// Useful for cleanup, to remove any possible analog source
#[derive(Bundle)]
pub struct AnalogSourcesCleanup {
    motion: AnalogSourceMouseMotion,
    scroll: AnalogSourceMouseScroll,
    stick: AnalogSourceGamepadStick,
    z: AnalogSourceGamepadZ,
}

impl<'a> From<&'a String> for InputActionName {
    fn from(value: &'a String) -> Self {
        Self(value.to_owned())
    }
}

impl<'a> From<&'a String> for InputAnalogName {
    fn from(value: &'a String) -> Self {
        Self(value.to_owned())
    }
}

impl<'a> From<&'a String> for InputActionBundle {
    fn from(value: &'a String) -> Self {
        Self {
            marker: InputAction,
            name: value.into(),
        }
    }
}

impl<'a> From<&'a String> for InputAnalogBundle {
    fn from(value: &'a String) -> Self {
        Self {
            marker: InputAnalog,
            name: value.into(),
        }
    }
}

impl<'a> From<&'a str> for InputActionName {
    fn from(value: &'a str) -> Self {
        Self(value.to_owned())
    }
}

impl<'a> From<&'a str> for InputAnalogName {
    fn from(value: &'a str) -> Self {
        Self(value.to_owned())
    }
}

impl<'a> From<&'a str> for InputActionBundle {
    fn from(value: &'a str) -> Self {
        Self {
            marker: InputAction,
            name: value.into(),
        }
    }
}

impl<'a> From<&'a str> for InputAnalogBundle {
    fn from(value: &'a str) -> Self {
        Self {
            marker: InputAnalog,
            name: value.into(),
        }
    }
}
