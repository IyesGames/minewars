use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<Gfx3dImpl>(SETTINGS_LOCAL.as_ref());
}

#[derive(Reflect, Default, Clone, PartialEq, Eq)]
#[reflect(Setting)]
pub enum Gfx3dImpl {
    Simple3D,
    #[default]
    Bespoke3D,
}

impl Setting for Gfx3dImpl {}
