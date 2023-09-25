use mw_app::map::NeedsMapSet;

use crate::{prelude::*, settings::MwRenderer};

pub mod camera;
pub mod tilemap;
pub mod sprites;

pub struct Gfx2dPlugin;

impl Plugin for Gfx2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            camera::Gfx2dCameraPlugin,
            sprites::Gfx2dSpritesPlugin,
            tilemap::Gfx2dTilemapPlugin,
        ));
        app.configure_sets(Update, (
            Gfx2dSet::Any.in_set(NeedsMapSet).run_if(rc_gfx2d_any),
            Gfx2dSet::Sprites.in_set(NeedsMapSet).run_if(rc_gfx2d_sprites),
            Gfx2dSet::Tilemap.in_set(NeedsMapSet).run_if(rc_gfx2d_tilemap),
        ));
    }
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gfx2dSet {
    Any,
    Sprites,
    Tilemap,
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Gfx2dTileSetupSet;

#[derive(Resource)]
pub struct TilemapInitted;

fn rc_gfx2d_any(
    settings: Option<Res<AllSettings>>,
) -> bool {
    settings.map(|s| s.renderer == MwRenderer::Tilemap || s.renderer == MwRenderer::Sprites).unwrap_or(false)
}

fn rc_gfx2d_sprites(
    settings: Option<Res<AllSettings>>,
) -> bool {
    settings.map(|s| s.renderer == MwRenderer::Sprites).unwrap_or(false)
}

fn rc_gfx2d_tilemap(
    settings: Option<Res<AllSettings>>,
) -> bool {
    settings.map(|s| s.renderer == MwRenderer::Tilemap).unwrap_or(false)
}

mod sprite {
    pub const WIDTH6: f32 = 112.0;
    pub const HEIGHT6: f32 = 128.0;

    pub const WIDTH4: f32 = 112.0;
    pub const HEIGHT4: f32 = 112.0;

    pub const STRIDE: usize = 10;

    pub const DIG: usize = 0 * STRIDE;
    pub const DIGSTAR: usize = 1 * STRIDE;
    pub const DIGEXTRA: usize = 8 * STRIDE;
    pub const TILES6: usize = 2 * STRIDE;
    pub const TILES4: usize = 3 * STRIDE;
    pub const FLAGS: usize = 7 * STRIDE;
    pub const OVERLAYS: usize = 6 * STRIDE;
    pub const GENTS: usize = 4 * STRIDE;
    pub const BRIDGE_DIAG: usize = 5 * STRIDE + 0;
    pub const BRIDGE_ANTIDIAG: usize = 5 * STRIDE + 1;
    pub const BRIDGE_HORIZ: usize = 5 * STRIDE + 2;
    pub const BRIDGE_VERT: usize = 5 * STRIDE + 3;
    pub const TILE_CURSOR: usize = 0;
    pub const TILE_WATER: usize = 1;
    pub const TILE_LAND: usize = 2;
    pub const TILE_FERTILE: usize = 3;
    pub const TILE_MTN: usize = 4;
    pub const TILE_FOREST: usize = 5;
    pub const TILE_DEAD: usize = 6;
    pub const TILE_DEADSKULL: usize = 7;
    pub const GENT_MINE: usize = GENTS + 0;
    pub const GENT_DECOY: usize = GENTS + 1;
    pub const GENT_FLASH: usize = GENTS + 2;
    pub const GENT_MINEACT: usize = GENTS + 3;
    pub const GENT_CIT: usize = GENTS + 3;
    pub const GENT_TOWER: usize = GENTS + 4;
    pub const GENT_WALL: usize = GENTS + 5;
    pub const EXPLOSION_MINE: usize = OVERLAYS + 0;
    pub const EXPLOSION_DECOY: usize = OVERLAYS + 1;
    pub const SMOKE: usize = OVERLAYS + 2;
    pub const STRIKE: usize = OVERLAYS + 3;
    pub const REVEAL: usize = OVERLAYS + 4;
}
