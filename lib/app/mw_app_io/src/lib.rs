/// Convenience, to be imported in every file in the crate
pub mod prelude {
    pub use mw_app_core::prelude::*;
}

use crate::prelude::*;

pub mod offline_host;
pub mod net;
pub mod mwfile;

pub mod settings;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        settings::plugin,
        net::plugin,
    ));
}
