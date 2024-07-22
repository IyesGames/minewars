use crate::prelude::*;

pub mod multilayer_image;

pub fn plugin(app: &mut App) {
    app.configure_stage_set_no_rc(Update, WidgetsUiUpdateSS);
    app.add_plugins((
        multilayer_image::plugin,
    ));
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetsUiUpdateSS;
