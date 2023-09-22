use crate::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(Update, GameInputSet.run_if(rc_accepting_game_input));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameInputSet;

/// If any entities with this component exist, gameplay input handling is suspended
#[derive(Component)]
pub struct InhibitGameInput;

/// Bevy run condition for our game input systems
///
/// Disables game input if input should be going elsewhere (such as UI).
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
