use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

#[derive(Component, Default, PartialEq, Eq)]
pub enum Gfx2dImpl {
    #[cfg_attr(not(feature = "tilemap"), default)]
    Sprites,
    #[cfg(feature = "tilemap")]
    #[cfg_attr(feature = "tilemap", default)]
    Tilemap,
}
