pub mod prelude {
    pub use mw_app_core::prelude::*;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

use crate::prelude::*;

pub(crate) mod assets;
pub(crate) mod settings;

mod root;
mod console;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::assets::plugin,
        crate::settings::plugin,
        crate::root::plugin,
        crate::console::plugin,
    ));
}
