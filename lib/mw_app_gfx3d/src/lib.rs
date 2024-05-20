#![allow(unused_variables)]

/// Convenience, to be imported in every file in the crate
pub mod prelude {
    pub use bevy::utils::{Duration, Instant};
    pub use iyes_bevy_extras::prelude::*;
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use mw_common::prelude::*;
    pub use mw_app_core::prelude::*;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

use crate::prelude::*;

pub(crate) mod assets;
pub(crate) mod misc;
pub(crate) mod settings;

pub(crate) mod asset_resolver;
pub(crate) mod map;

pub(crate) mod camera;
pub(crate) mod simple;
pub(crate) mod bespoke;

pub mod ui;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::assets::plugin,
        crate::misc::plugin,
        crate::settings::plugin,
        crate::asset_resolver::plugin,
        crate::map::plugin,
        crate::camera::plugin,
        crate::simple::plugin,
        crate::bespoke::plugin,
        crate::ui::plugin,
    ));
}
