#![feature(trait_upcasting)]

#![allow(unused_variables)]

/// Convenience, to be imported in every file in the crate
pub mod prelude {
    pub use bevy::utils::{Duration, Instant};
    pub use iyes_bevy_extras::prelude::*;
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use mw_common::prelude::*;
    pub use modular_bitfield::prelude::*;
    pub use crate::apporg::*;
    pub use crate::settings::prelude::*;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

// foundational
pub mod apporg;
pub mod assets;
pub mod settings;
pub mod ui;
pub mod value;

// governors and game state
pub mod driver;
pub mod map;
pub mod session;
pub mod player;
pub mod user;
pub mod view;

// support for client-side features
pub mod camera;
pub mod haptic;
pub mod locale;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    // external plugins
    app.add_plugins((
        bevy_fluent::FluentPlugin,
        bevy_tweening::TweeningPlugin,
    ));
    // our plugins
    app.add_plugins((
        crate::apporg::plugin,
        crate::assets::plugin,
        crate::camera::plugin,
        crate::driver::plugin,
        crate::haptic::plugin,
        crate::locale::plugin,
        crate::map::plugin,
        crate::player::plugin,
        crate::session::plugin,
        crate::settings::plugin,
        crate::ui::plugin,
        crate::user::plugin,
        crate::value::plugin,
        crate::view::plugin,
    ));
}
