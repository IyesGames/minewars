use crate::prelude::*;
use super::*;

#[cfg(feature = "gfx2d")]
mod gfx2d;

pub struct TouchInputPlugin;

impl Plugin for TouchInputPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "gfx2d")]
        app.add_plugins(gfx2d::Gfx2dTouchInputPlugin);
    }
}
