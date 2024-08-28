#![feature(btree_extract_if)]

/// Convenience, to be imported in every file in the crate
pub mod prelude {
    pub use mw_app_core::prelude::*;
}

pub mod cli;
pub mod offline_host;
pub mod mwfile;

pub mod settings;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::cli::plugin,
        crate::settings::plugin,
    ));
}
