pub mod prelude {
    pub use mw_app::prelude::*;
    pub use crate::PROPRIETARY;
}

pub const PROPRIETARY: bool = cfg!(feature = "proprietary");

use crate::prelude::*;

pub fn plugin(app: &mut App) {
}
