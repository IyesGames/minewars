use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<Gfx2dImpl>(SETTINGS_LOCAL.as_ref());
}

#[derive(Reflect, Default, Clone, PartialEq, Eq)]
#[reflect(Setting)]
pub enum Gfx2dImpl {
    Sprites,
    #[default]
    Bespoke,
    #[cfg(feature = "tilemap")]
    Tilemap,
}

impl Setting for Gfx2dImpl {}
