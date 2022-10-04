use mw_common::game::{MapDescriptor, TileKind, MineKind};
use mw_common::grid::*;

use crate::prelude::*;

use crate::assets::TileAssets;
use crate::camera::{GridCursor, translation_pos, ZoomLevel};
use crate::settings::PlayerPaletteSettings;

use super::*;

mod sprites;
mod tilemap;

#[derive(Component)]
struct CursorSprite;
#[derive(Component)]
struct BaseSprite;
#[derive(Component)]
struct DigitSprite;
#[derive(Component)]
struct GentSprite;
#[derive(Component)]
struct MineSprite;
#[derive(Component)]
struct CitSprite;
#[derive(Component)]
struct TowerSprite;
#[derive(Component)]
struct FortSprite;

#[derive(Component)]
struct MineActiveAnimation {
    timer: Timer,
}

#[derive(Component)]
struct ExplosionSprite {
    timer: Timer,
}

/// Reference to a sprite entity displaying the minesweeper digit, if any
#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileDigitSprite(Entity);
/// Reference to a sprite entity displaying a mine, if any
#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileMineSprite(Entity);

pub struct MapGfx2dPlugin;

impl Plugin for MapGfx2dPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppGlobalState::InGame, setup_cursor);
        app.add_system(
            cursor_sprite
                .run_in_state(AppGlobalState::InGame)
                .after("cursor")
        );
        app.add_plugin(sprites::MapGfxSpritesPlugin);
        app.add_plugin(tilemap::MapGfxTilemapPlugin);
    }
}

#[derive(Bundle)]
struct CursorSpriteBundle {
    #[bundle]
    sprite: SpriteBundle,
    pos: TilePos,
    marker: CursorSprite,
    cleanup: MapCleanup,
}

fn setup_cursor(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    zoom: Res<ZoomLevel>,
) {
    commands.spawn_bundle(CursorSpriteBundle {
        sprite: SpriteBundle {
            sprite: Sprite {
                rect: Some(tileid::get_rect(zoom.desc.size, tileid::tiles::CURSOR)),
                ..Default::default()
            },
            texture: tiles.tiles6[0].clone(),
            transform: Transform::from_xyz(0.0, 0.0, zpos::CURSOR),
            ..Default::default()
        },
        pos: Pos::origin().into(),
        marker: CursorSprite,
        cleanup: MapCleanup,
    });
}

fn cursor_sprite(
    mut q: Query<(&mut Transform, &mut TilePos), With<CursorSprite>>,
    crs: Res<GridCursor>,
    zoom: Res<ZoomLevel>,
    descriptor: Res<MapDescriptor>,
) {
    if !crs.is_changed() {
        return;
    }
    let (mut xf, mut pos) = q.single_mut();
    *pos = crs.0.into();
    xf.translation = translation_pos(descriptor.topology, crs.0, &zoom.desc).extend(zpos::CURSOR);
}

/// Generate fancy alpha values for water
fn fancytint<C: CompactMapCoordExt>(map_size: u8, c: C, f_kind: impl Fn(Pos) -> TileKind) -> f32 {
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
            } else if f_kind(c2.into()) != TileKind::Water {
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

mod zpos {
    pub const CURSOR: f32 = 10.0;
    pub const EXPLOSION: f32 = 5.0;
    pub const DIGIT: f32 = 3.0;
    pub const GENTS: f32 = 2.0;
    pub const ROAD: f32 = 1.0;
}

pub mod tileid {
    #![allow(dead_code)]

    use crate::prelude::*;
    use bevy::sprite::Rect;

    pub mod tiles {
        pub const CURSOR: u32 = 0;
        pub const WATER: u32 = 1;
        pub const LAND: u32 = 2;
        pub const MTN: u32 = 3;
        pub const FERTILE: u32 = 4;
        pub const DEGEN: u32 = 5;
        pub const SKULL: u32 = 6;
    }

    pub mod gents {
        pub const MINE: u32 = 0;
        pub const DECOY: u32 = 1;
        pub const EXPLODE_MINE: u32 = 2;
        pub const EXPLODE_DECOY: u32 = 3;
        pub const MINE_ACTIVE: u32 = 4;
        pub const CIT: u32 = 5;
        pub const TOWER: u32 = 6;
        pub const FORT: u32 = 7;
    }

    pub fn get_rect(size: u32, id: u32) -> Rect {
        Rect {
            min: Vec2::new(0.0, id as f32 * size as f32),
            max: Vec2::new(size as f32, (id + 1) as f32 * size as f32),
        }
    }
}
