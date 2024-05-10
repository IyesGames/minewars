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

pub(crate) mod camera;
pub(crate) mod sprites;
#[cfg(feature = "tilemap")]
pub(crate) mod tilemap;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        #[cfg(feature = "tilemap")]
        bevy_ecs_tilemap::TilemapPlugin,
    ));
    app.add_plugins((
        crate::assets::plugin,
        crate::misc::plugin,
        crate::settings::plugin,
        crate::camera::plugin,
        crate::sprites::plugin,
        #[cfg(feature = "tilemap")]
        crate::tilemap::plugin,
    ));
}
