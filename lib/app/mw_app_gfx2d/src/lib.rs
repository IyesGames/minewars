#![allow(unused_variables)]

/// Convenience, to be imported in every file in the crate
pub mod prelude {
    pub use mw_app_core::prelude::*;
}

use crate::prelude::*;

pub(crate) mod assets;
pub(crate) mod misc;
pub(crate) mod settings;

pub(crate) mod camera;
pub(crate) mod sprites;
pub(crate) mod bespoke;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::assets::plugin,
        crate::misc::plugin,
        crate::settings::plugin,
        crate::camera::plugin,
        crate::sprites::plugin,
        crate::bespoke::plugin,
    ));
}
