use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

#[derive(Component, Default, PartialEq, Eq)]
pub enum Gfx3dImpl {
    Simple3D,
    #[default]
    Bespoke3D,
}
