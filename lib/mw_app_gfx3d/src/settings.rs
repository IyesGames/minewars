use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting_entity::<Gfx3dImpl>(SETTINGS_LOCAL.as_ref());
}

#[derive(Component, Default, Clone, PartialEq, Eq)]
#[derive(Reflect)]
pub enum Gfx3dImpl {
    Simple3D,
    #[default]
    Bespoke3D,
}

impl GovernorSetting for Gfx3dImpl {}
