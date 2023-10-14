pub mod prelude {
    pub use mw_common::prelude::*;
    pub use iyes_bevy_extras::prelude::*;
    pub use crate::appstate::*;
    pub use crate::settings::{AllSettings, NeedsSettingsSet};
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

pub mod appstate;
pub mod camera;
pub mod map;
pub mod player;
pub mod view;
pub mod settings;

pub mod bevyhost;

use crate::prelude::*;

pub struct MwCommonPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GameEventSet;

impl Plugin for MwCommonPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<mw_common::game::event::GameEvent>();
        app.add_plugins((
            appstate::AppStatesPlugin,
            camera::MwCameraPlugin,
            settings::SettingsPlugin,
            map::MapPlugin,
            view::GameViewPlugin,
        ));
    }
}
