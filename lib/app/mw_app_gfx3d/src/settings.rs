use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.init_setting::<Gfx3dImpl>(SETTINGS_LOCAL.as_ref());
    app.init_setting::<Camera3dSettings>(SETTINGS_USER.as_ref());
}

#[derive(Reflect, Default, Clone, PartialEq, Eq)]
#[reflect(Setting)]
pub enum Gfx3dImpl {
    Simple,
    #[default]
    Bespoke,
}

impl Setting for Gfx3dImpl {}

#[derive(Reflect, Clone)]
#[reflect(Setting)]
pub struct Camera3dSettings {
    pub fov: f32,
}

impl Default for Camera3dSettings {
    fn default() -> Self {
        Self {
            fov: 45.0,
        }
    }
}

impl Setting for Camera3dSettings {}
