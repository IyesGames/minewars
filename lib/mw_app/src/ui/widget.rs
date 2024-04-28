use crate::prelude::*;

pub mod form;
pub mod textfield;

pub fn plugin(app: &mut App) {
    app.add_plugins(textfield::plugin);
}
