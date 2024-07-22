pub mod prelude {
    pub use mw_app_core::prelude::*;
}

pub(crate) mod assets;
pub(crate) mod settings;

mod root;
mod console;

mod scoreboard;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::assets::plugin,
        crate::settings::plugin,
        crate::root::plugin,
        crate::console::plugin,
        crate::scoreboard::plugin,
    ));
}
