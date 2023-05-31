use crate::prelude::*;

pub struct CliPlugin;

impl Plugin for CliPlugin {
    fn build(&self, app: &mut App) {
        app.register_clicommand_noargs("exit", exit);
        app.register_clicommand_noargs("settings_reload", settings_reload);
        app.register_clicommand_noargs("settings_write", settings_write);
        #[cfg(feature = "dev")]
        app.register_clicommand_noargs("devmode", enter_devmode);
    }
}

fn exit(mut evw_exit: EventWriter<bevy::app::AppExit>) {
    evw_exit.send(bevy::app::AppExit);
}

#[cfg(feature = "dev")]
fn enter_devmode(
    mut appstate: ResMut<NextState<AppState>>,
) {
    appstate.set(AppState::InGame);
}

fn settings_reload(world: &mut World) {
    world.remove_resource::<crate::settings::SettingsLoaded>();
}

fn settings_write(world: &World) {
    crate::settings::write_settings(world);
}
