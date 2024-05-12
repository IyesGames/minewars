use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting_entity::<Gfx2dImpl>(SETTINGS_LOCAL.as_ref());
}

#[derive(Component, Default, Clone, PartialEq, Eq)]
#[derive(Reflect)]
pub enum Gfx2dImpl {
    #[cfg_attr(not(feature = "tilemap"), default)]
    Sprites,
    #[cfg(feature = "tilemap")]
    #[cfg_attr(feature = "tilemap", default)]
    Tilemap,
}

impl GovernorSetting for Gfx2dImpl {}
