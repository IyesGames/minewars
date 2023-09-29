use bevy::input::{mouse::MouseButtonInput, ButtonState};

use crate::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MouseClick>();
        app.configure_set(Update, GameInputSet.run_if(rc_accepting_game_input));
        app.add_systems(PreUpdate, mouse_click.run_if(in_state(AppState::InGame)));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameInputSet;

/// If any entities with this component exist, gameplay input handling is suspended
#[derive(Component)]
pub struct InhibitGameInput;

#[derive(Event)]
pub struct MouseClick(pub MouseButton);

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

fn mouse_click(
    settings: Option<Res<AllSettings>>,
    time: Res<Time>,
    mut evr: EventReader<MouseButtonInput>,
    mut evw: EventWriter<MouseClick>,
    mut map: Local<HashMap<MouseButton, Duration>>,
) {
    let millis_click = settings.map(|s| s.input.millis_click).unwrap_or(250);
    for ev in evr.iter() {
        match ev.state {
            ButtonState::Pressed => {
                map.insert(ev.button, time.elapsed());
            },
            ButtonState::Released => {
                if let Some(start) = map.remove(&ev.button) {
                    let delta = time.elapsed() - start;
                    if delta.as_millis() <= millis_click as u128 {
                        evw.send(MouseClick(ev.button));
                    }
                }
            },
        }
    }
}
