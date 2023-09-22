pub mod prelude {
    pub use mw_common::prelude::*;
    pub use iyes_bevy_extras::prelude::*;
    pub use crate::appstate::*;
}

pub mod appstate;

pub mod map;
pub mod player;

pub mod bevyhost;

use crate::prelude::*;

pub struct MwCommonPlugin;

impl Plugin for MwCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            appstate::AppStatesPlugin,
            map::MapPlugin,
        ));
    }
}
