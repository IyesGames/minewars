use crate::prelude::*;

pub struct ErrorPlugin;

impl Plugin for ErrorPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ErrorEvent>();
    }
}

pub enum ErrorEvent {
    Normal(AnyError),
    Critical(AnyError),
}

fn spawn_error_dialogs(
    mut commands: Commands,
) {
}
