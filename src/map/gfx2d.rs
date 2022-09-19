use mw_common::game::{MapDescriptor, TileKind, MineKind};
use mw_common::grid::*;

use crate::prelude::*;

use crate::assets::TileAssets;
use crate::camera::GridCursor;
use crate::settings::PlayerPaletteSettings;

use super::*;

mod sprites;
mod tilemap;

#[derive(Component)]
struct CursorSprite;
#[derive(Component)]
struct BaseSprite;
#[derive(Component)]
struct DecalSprite;
#[derive(Component)]
struct DigitSprite;
#[derive(Component)]
struct MineSprite;
#[derive(Component)]
struct CitSprite;

#[derive(Component)]
struct MineActiveAnimation {
    timer: Timer,
}

#[derive(Component)]
struct ExplosionSprite {
    timer: Timer,
}

/// Reference to a sprite entity displaying a "decal", if any
#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileDecalSprite(Entity);
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

fn setup_cursor(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    descriptor: Res<MapDescriptor>,
) {
    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            rect: Some(tileid::get_rect(256, tileid::tiles::CURSOR)),
            ..Default::default()
        },
        texture: tiles.tiles6[0].clone(),
        transform: Transform::from_xyz(0.0, 0.0, zpos::CURSOR),
        ..Default::default()
    })
        .insert(CursorSprite)
        .insert(MapCleanup);
}

fn cursor_sprite(
    mut q: Query<&mut Transform, With<CursorSprite>>,
    crs: Res<GridCursor>,
    descriptor: Res<MapDescriptor>,
) {
    let mut xf = q.single_mut();
    xf.translation = translation_pos(descriptor.topology, crs.0).extend(zpos::CURSOR);
}

fn translation_pos(topology: Topology, pos: Pos) -> Vec2 {
    match topology {
        Topology::Hex => {
            Hex::from(pos).translation() * Vec2::new(224.0, 256.0)
        }
        Topology::Sq | Topology::Sqr => {
            Sq::from(pos).translation() * Vec2::new(224.0, 224.0)
        }
    }
}

mod zpos {
    pub const CURSOR: f32 = 10.0;
    pub const EXPLOSION: f32 = 5.0;
    pub const DIGIT: f32 = 3.0;
    pub const GENTS: f32 = 2.0;
    pub const ROADS: f32 = 1.0;
    pub const BASE: f32 = 0.0;
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
