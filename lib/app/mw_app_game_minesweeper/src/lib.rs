pub mod prelude {
    pub use mw_app_core::prelude::*;
}

mod cli;
mod settings;
mod input;
mod map;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::cli::plugin,
        crate::input::plugin,
        crate::map::plugin,
        crate::settings::plugin,
    ));
}
