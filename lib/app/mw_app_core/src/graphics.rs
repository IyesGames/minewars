use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.register_type::<GraphicsStyle>();
}

#[derive(Bundle)]
pub struct GraphicsGovernorBundle {
    pub cleanup: GamePartialCleanup,
    pub marker: GraphicsGovernor,
    pub style: CurrentGraphicsStyle,
}

#[derive(Component)]
pub struct GraphicsGovernor;

#[derive(Component)]
pub struct Gfx2dEnabled;

#[derive(Component)]
pub struct Gfx3dEnabled;

#[derive(Component)]
pub struct CurrentGraphicsStyle(pub GraphicsStyle);

#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphicsStyle {
    Gfx2d,
    Gfx3d,
}
