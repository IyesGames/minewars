pub mod prelude {
    pub use mw_engine::prelude::*;
}

pub mod assets;
pub mod camera;
pub mod ninepatch;
pub mod root;
pub mod widgets;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        crate::assets::plugin,
        crate::camera::plugin,
        crate::ninepatch::plugin,
        crate::root::plugin,
        crate::widgets::plugin,
    ));
}
