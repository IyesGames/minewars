use crate::prelude::*;

pub mod multilayer_image;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        multilayer_image::plugin,
    ));
}
