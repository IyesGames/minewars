use crate::prelude::*;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(iyes_bevy_util::ui::init_camera);
    }
}
