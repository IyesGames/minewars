/// Convenience, to be imported in every file in the crate
pub mod prelude {
    pub use bevy::utils::{Duration, Instant};
    pub use iyes_bevy_extras::prelude::*;
    pub use iyes_progress::prelude::*;
    pub use iyes_cli::prelude::*;
    pub use mw_common::prelude::*;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

use crate::prelude::*;

pub fn plugin(app: &mut App) {
}
