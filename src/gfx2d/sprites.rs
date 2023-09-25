use crate::assets::GameAssets;
use crate::prelude::*;

use mw_common::grid::*;
use mw_common::game::*;
use mw_app::map::*;

use super::Gfx2dSet;
use super::Gfx2dTileSetupSet;
use super::TilemapInitted;

pub struct Gfx2dSpritesPlugin;

impl Plugin for Gfx2dSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            (
                setup_base_tile::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                setup_base_tile::<Sq>
                    .in_set(MapTopologySet(Topology::Sq)),
            )
                .in_set(Gfx2dTileSetupSet)
                .run_if(not(resource_exists::<TilemapInitted>())),
            (
                tile_kind,
                tile_owner,
            )
                .run_if(resource_exists::<TilemapInitted>()),
        ).in_set(Gfx2dSet::Sprites));
    }
}

fn setup_base_tile<C: Coord>(
    world: &mut World,
) {
    let texture_atlas = world.resource::<GameAssets>().sprites.clone();
    let index = world.remove_resource::<MapTileIndex<C>>().unwrap();
    let (sprite_index, width, height) = match C::TOPOLOGY {
        Topology::Hex => (
            super::sprite::TILES6 + super::sprite::TILE_WATER,
            super::sprite::WIDTH6, super::sprite::HEIGHT6,
        ),
        Topology::Sq => (
            super::sprite::TILES4 + super::sprite::TILE_WATER,
            super::sprite::WIDTH4, super::sprite::HEIGHT4,
        ),
    };
    for (c, &e) in index.0.iter() {
        let trans = c.translation();
        world.entity_mut(e).insert(SpriteSheetBundle {
            texture_atlas: texture_atlas.clone(),
            sprite: TextureAtlasSprite {
                color: Color::WHITE,
                index: sprite_index,
                ..Default::default()
            },
            transform: Transform::from_xyz(trans.x * width, trans.y * height, 0.0),
            ..Default::default()
        });
    }
    world.insert_resource(index);
    world.insert_resource(TilemapInitted);
}

fn tile_kind(
    desc: Res<MapDescriptor>,
    mut q: Query<(&mut TextureAtlasSprite, &TileKind), Changed<TileKind>>,
) {
    let i_base = match desc.topology {
        Topology::Hex => super::sprite::TILES6,
        Topology::Sq => super::sprite::TILES4,
    };

    for (mut spr, kind) in &mut q {
        spr.index = i_base + match kind {
            TileKind::Water => super::sprite::TILE_WATER,
            TileKind::Foundation => super::sprite::TILE_LAND,
            TileKind::Regular => super::sprite::TILE_LAND,
            TileKind::Fertile => super::sprite::TILE_FERTILE,
            TileKind::Forest => super::sprite::TILE_FOREST,
            TileKind::Mountain => super::sprite::TILE_MTN,
            TileKind::Destroyed => super::sprite::TILE_DEAD,
        }
    }
}

fn tile_owner(
    settings: Res<AllSettings>,
    mut q: Query<(&mut TextureAtlasSprite, &TileOwner), Changed<TileOwner>>,
) {
    for (mut spr, owner) in &mut q {
        spr.color = settings.player_colors.visible[owner.0.i()].into();
    }
}
