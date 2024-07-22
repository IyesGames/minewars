use crate::prelude::*;

pub mod properties;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        self::properties::plugin,
    ));
}
