use bevy::ecs::system::SystemId;

use crate::{locale::L10nKey, prelude::*};

pub fn plugin(app: &mut App) {
    app.register_type::<InputActionName>();
}

/// Stage Sets related to input handling
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameInputSS {
    Detect,
    Manage,
    Handle,
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
    pub callback: InputActionCallback,
    pub ui: InputActionUi,
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
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputActionName(pub &'static str);

/// How to represent this action in UIs
#[derive(Component)]
pub struct InputActionUi {
    pub l10n_name: L10nKey,
    pub l10n_tooltip: L10nKey,
    pub icon_bg: Option<Handle<Image>>,
    pub icon_fg: Option<Handle<Image>>,
}

/// Systems to run for a given action
#[derive(Component)]
pub struct InputActionCallback {
    /// When the key/button that triggers the action is just pressed
    pub on_press: Option<SystemId>,
    /// When the key/button that triggers the action is just released
    pub on_release: Option<SystemId>,
}

/// Bundle for entities representing Analog Controls
///
/// When an analog action is being performed, an
/// `InputAnalogActive` component will be inserted,
/// indicating the source of the analog values
/// (which input device is performing the action).
#[derive(Bundle)]
pub struct InputAnalogBundle {
    pub marker: InputAnalog,
    pub name: InputAnalogName,
    pub callback: InputAnalogCallback,
}

/// Marker for Analog Inputs
#[derive(Component)]
pub struct InputAnalog;

/// Is the Analog Input allowed to be used?
#[derive(Component)]
pub struct InputAnalogEnabled;

/// Internal ID for the analog input.
///
/// Used to enable use cases like storing settings.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InputAnalogName(pub &'static str);

/// Is the analog action currently being performed?
/// What device is controlling/performing it?
#[derive(Component)]
pub struct InputAnalogActive {
    pub source: AnalogSource,
}

/// Systems to run for a given analog
#[derive(Component)]
pub struct InputAnalogCallback {
    /// When the analog is initiated
    pub on_start: Option<SystemId>,
    /// When the analog is completed
    pub on_stop: Option<SystemId>,
}

/// Where does analog data come from?
///
/// Some of these are 2D (X/Y), some 1D. It is up the
/// implementations of different input mechanics to
/// decide how to deal with that.
#[derive(Reflect, Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum AnalogSource {
    /// Either relative mouse motion, or pointer/cursor position,
    /// whichever is more appropriate is an implementation detail.
    MouseMotion,
    /// Scrolling / mouse wheel
    MouseScroll,
    GamepadLeftStick(Gamepad),
    GamepadRightStick(Gamepad),
    GamepadAnyStick(Gamepad),
    GamepadLeftZ(Gamepad),
    GamepadRightZ(Gamepad),
}
