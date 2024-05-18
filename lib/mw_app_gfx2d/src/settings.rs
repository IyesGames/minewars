use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<Gfx2dImpl>(SETTINGS_LOCAL.as_ref());
}

#[derive(Reflect, Default, Clone, PartialEq, Eq)]
#[reflect(Setting)]
pub enum Gfx2dImpl {
    #[cfg_attr(not(feature = "tilemap"), default)]
    Sprites,
    #[cfg(feature = "tilemap")]
    #[cfg_attr(feature = "tilemap", default)]
    Tilemap,
}

impl Setting for Gfx2dImpl {}
