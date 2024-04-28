use crate::prelude::*;
use crate::map::NeedsMapSet;
use crate::settings::MwRenderer;

pub mod asset_resolver;
pub mod camera;
pub mod map;
pub mod simple3d;

pub fn plugin(app: &mut App) {
    app.add_plugins((
        asset_resolver::plugin,
        camera::plugin,
        map::plugin,
        simple3d::plugin,
    ));
    app.configure_sets(Update, (
        Gfx3dModeSet::Any.in_set(NeedsMapSet).run_if(rc_gfx3d_any),
        Gfx3dModeSet::Simple3D.in_set(NeedsMapSet).run_if(rc_gfx3d_simple3d),
    ));
    app.configure_sets(OnEnter(AppState::InGame), (
        Gfx3dModeSet::Any.run_if(rc_gfx3d_any),
        Gfx3dModeSet::Simple3D.run_if(rc_gfx3d_simple3d),
    ));
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gfx3dModeSet {
    Any,
    Simple3D,
}

fn rc_gfx3d_any(
    settings: Option<Res<AllSettings>>,
) -> bool {
    settings.map(|s| s.renderer == MwRenderer::Simple3D).unwrap_or(false)
}

fn rc_gfx3d_simple3d(
    settings: Option<Res<AllSettings>>,
) -> bool {
    settings.map(|s| s.renderer == MwRenderer::Simple3D).unwrap_or(false)
}

const TILE_SCALE: f32 = 64.0;
const RENDER_RANGE: f32 = 8192.0;
