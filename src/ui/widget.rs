use crate::prelude::*;

pub mod form;
pub mod textfield;

pub(super) struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(textfield::TextFieldPlugin);
    }
}
