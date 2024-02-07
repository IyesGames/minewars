use mw_app::map::NeedsMapSet;
use mw_app::settings::MwRenderer;

use crate::prelude::*;

pub mod asset_resolver;
pub mod camera;
pub mod map;
pub mod simple3d;

pub struct Gfx3dPlugin;

impl Plugin for Gfx3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            asset_resolver::Gfx3dAssetResolverPlugin,
            camera::Gfx3dCameraPlugin,
            map::Gfx3dMapPlugin,
            simple3d::Gfx3dSimple3dPlugin,
        ));
        app.configure_sets(Update, (
            Gfx3dSet::Any.in_set(NeedsMapSet).run_if(rc_gfx3d_any),
            Gfx3dSet::Simple3D.in_set(NeedsMapSet).run_if(rc_gfx3d_simple3d),
        ));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gfx3dSet {
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
