use mw_app_core::graphics::{Gfx2dEnabled, GraphicsGovernor};
use mw_common::{game::TileKind, grid::*};

use crate::{prelude::*, settings::Gfx2dImpl};

pub fn plugin(app: &mut App) {
    app.configure_sets(Update, (
        Gfx2dImplSet::Any
            .run_if(any_filter::<(With<GraphicsGovernor>, With<Gfx2dEnabled>)>),
        Gfx2dImplSet::Sprites
            .in_set(Gfx2dImplSet::Any)
            .run_if(rc_gfx2d_sprites),
        Gfx2dImplSet::Bespoke
            .in_set(Gfx2dImplSet::Any)
            .run_if(rc_gfx2d_bespoke),
    ));
    app.configure_sets(OnEnter(AppState::InGame), (
        Gfx2dImplSet::Any
            .run_if(any_filter::<(With<GraphicsGovernor>, With<Gfx2dEnabled>)>),
        Gfx2dImplSet::Sprites
            .in_set(Gfx2dImplSet::Any)
            .run_if(rc_gfx2d_sprites),
        Gfx2dImplSet::Bespoke
            .in_set(Gfx2dImplSet::Any)
            .run_if(rc_gfx2d_bespoke),
    ));
}

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Gfx2dImplSet {
    Any,
    Sprites,
    Bespoke,
}

#[derive(Component)]
pub struct CursorSprite;
#[derive(Component)]
pub struct BaseSprite;
#[derive(Component)]
pub struct DigitSprite;
#[derive(Component)]
pub struct GentSprite;
#[derive(Component)]
pub struct RegHighlightSprite;
#[derive(Component)]
pub struct ExplosionSprite {
    pub timer: Timer,
}

fn rc_gfx2d_sprites(
    settings: Settings,
) -> bool {
    settings.get::<Gfx2dImpl>().map(|s|
        *s == Gfx2dImpl::Sprites
    ).unwrap_or(false)
}

fn rc_gfx2d_bespoke(
    settings: Settings,
) -> bool {
    settings.get::<Gfx2dImpl>().map(|s|
        *s == Gfx2dImpl::Bespoke
    ).unwrap_or(false)
}

/// Generate fancy alpha values for water
pub fn fancytint<C: Coord>(map_size: u8, c: C, f_kind: impl Fn(C) -> TileKind) -> f32 {
    let mut d_edge = 0;
    let mut d_land = 0;

    'outer: for r in 1..=map_size {
        for c2 in c.iter_ring(r) {
            if c2.ring() > map_size {
                if d_edge == 0 {
                    d_edge = r;
                }
                if d_land != 0 {
                    break 'outer;
                }
            } else if f_kind(c2) != TileKind::Water {
                if d_land == 0 {
                    d_land = r;
                }
                if d_edge != 0 {
                    break 'outer;
                }
            }
        }
    }

    if d_land >= d_edge {
        0.0
    } else {
        let x = (d_edge - d_land) as f32 / d_edge as f32;
        x * x
    }
}

#[allow(dead_code)]
pub mod sprite {
    pub const WIDTH6: f32 = 224.0;
    pub const HEIGHT6: f32 = 256.0;

    pub const WIDTH4: f32 = 216.0;
    pub const HEIGHT4: f32 = 216.0;

    pub const TILES6: usize = 0;
    pub const TILES4: usize = 10;

    pub const DIG: usize = 0;
    pub const DIGSTAR: usize = 10;

    pub const EXPLOSION_MINE_START: usize = 0;
    pub const EXPLOSION_MINE_END: usize = 10;
    pub const EXPLOSION_DECOY_START: usize = 10;
    pub const EXPLOSION_DECOY_END: usize = 20;

    pub const TILE_CURSOR: usize = 0;
    pub const TILE_WATER: usize = 1;
    pub const TILE_LAND: usize = 2;
    pub const TILE_FERTILE: usize = 3;
    pub const TILE_MTN: usize = 4;
    pub const TILE_FOREST: usize = 5;
    pub const TILE_DEAD: usize = 6;
    pub const TILE_FOUNDATION: usize = 8;
    pub const TILE_HIGHLIGHT: usize = 9;

    pub const GENT_MINE: usize = 0;
    pub const GENT_DECOY: usize = 1;
    pub const GENT_FLASH: usize = 2;
    pub const GENT_MINEACT: usize = 3;
    pub const GENT_SMOKE: usize = 6;
    pub const GENT_SKULL: usize = 7;
    pub const GENT_CIT: usize = 8;
    pub const GENT_TOWER: usize = 9;
}

#[allow(dead_code)]
pub mod zpos {
    pub const TILE: f32 = 1.0;

    pub const CURSOR: f32 = TILE + 10.0;
    pub const OVERLAYS: f32 = TILE + 5.0;
    pub const DIGIT: f32 = TILE + 3.0;
    pub const GENTS: f32 = TILE + 2.0;
    pub const ROAD: f32 = TILE + 1.0;

    pub const REGHILIGHT: f32 = TILE - 1.0;
}
