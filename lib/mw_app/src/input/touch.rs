use crate::prelude::*;
use super::*;

#[cfg(feature = "gfx2d")]
mod gfx2d;

pub fn plugin(app: &mut App) {
    #[cfg(feature = "gfx2d")]
    app.add_plugins(gfx2d::plugin);
}
