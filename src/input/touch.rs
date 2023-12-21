use crate::prelude::*;
use super::*;

mod gfx2d;

pub struct TouchInputPlugin;

impl Plugin for TouchInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(gfx2d::Gfx2dTouchInputPlugin);
    }
}
