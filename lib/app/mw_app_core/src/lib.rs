#![allow(unused_variables)]

/// Convenience, to be imported in every file in the crate
pub mod prelude {
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use mw_common::prelude::*;
    pub use mw_engine::prelude::*;
    pub use modular_bitfield::prelude::*;
    pub use crate::apporg::*;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

// foundational
pub mod apporg;
pub mod assets;

// governors and game state
pub mod driver;
pub mod map;
pub mod session;
pub mod player;
pub mod user;
pub mod input;
pub mod graphics;

// support for client-side features
pub mod camera;
pub mod haptic;
pub mod locale;
pub mod view;
pub mod settings;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    // external plugins
    app.add_plugins((
        bevy_fluent::FluentPlugin,
    ));
    // our plugins
    app.add_plugins((
        crate::apporg::plugin,
        crate::assets::plugin,
    ));
    app.add_plugins((
        crate::camera::plugin,
        crate::driver::plugin,
        crate::graphics::plugin,
        crate::haptic::plugin,
        crate::input::plugin,
        crate::locale::plugin,
        crate::map::plugin,
        crate::player::plugin,
        crate::session::plugin,
        crate::user::plugin,
        crate::view::plugin,
        crate::settings::plugin,
    ));
}
