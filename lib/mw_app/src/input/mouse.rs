use crate::prelude::*;
use super::*;

pub struct MouseInputPlugin;

impl Plugin for MouseInputPlugin {
    fn build(&self, app: &mut App) {
    }
}

fn rc_accepting_mouse_input(
    q_inhibit: Query<(), With<InhibitGameInput>>,
    q_ui_interaction: Query<&Interaction>,
) -> bool {
    if !q_inhibit.is_empty() {
        return false;
    }
    let any_interaction = q_ui_interaction.iter().any(|i| *i != Interaction::None);
    !any_interaction
}

pub fn add_minewars_defaults(map: &mut InputMap<InputAction>) {
    map.insert(MouseButton::Left, InputAction::UseCurrentTool);
}
