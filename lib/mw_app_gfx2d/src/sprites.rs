use mw_app_core::map::*;
use mw_common::grid::*;

use crate::{assets::Gfx2dAssets, misc::*, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        setup_tile_entities
            .track_progress()
            .in_set(Gfx2dImplSet::Sprites)
            .run_if(any_filter::<With<MapGovernor>>),
    )
        .in_set(InStateSet(AppState::GameLoading)),
    );
    app.add_systems(OnEnter(AppState::InGame), (
        reveal_sprites_onenter_ingame,
    ));
}

#[derive(Component)]
pub struct MapSpritesIndex(pub MapDataPos<Entity>);

fn setup_tile_entities(
    mut commands: Commands,
    spreader: Res<WorkSpreader>,
    assets: Res<Gfx2dAssets>,
    q_map: Query<(Entity, &MapDescriptor, &MapTileIndex, Has<MapSpritesIndex>), With<MapGovernor>>,
) -> Progress {
    let (e_map, desc, tile_index) = match q_map.get_single() {
        Err(_) => return false.into(),
        Ok((_, _, _, true)) => return true.into(),
        Ok((e, desc, tile_index, false)) => (e, desc, tile_index),
    };
    if spreader.acquire() {
        return false.into();
    }

    let (base_i, width, height) = match desc.topology {
        Topology::Hex => (
            sprite::TILES6 + sprite::TILE_WATER,
            sprite::WIDTH6, sprite::HEIGHT6,
        ),
        Topology::Sq => (
            sprite::TILES4 + sprite::TILE_WATER,
            sprite::WIDTH4, sprite::HEIGHT4,
        ),
    };

    let mut sprites_index = MapSpritesIndex(
        MapDataPos::new(desc.size, Entity::PLACEHOLDER)
    );

    for (c, &e) in tile_index.0.iter() {
        let trans = match desc.topology {
            Topology::Hex => Hex::from(c).translation(),
            Topology::Sq => Sq::from(c).translation(),
        };
        let e = commands.spawn((
            BaseSprite,
            SpriteSheetBundle {
                texture: assets.sprites_img.clone(),
                atlas: TextureAtlas {
                    index: base_i,
                    layout: assets.sprites_layout.clone(),
                },
                sprite: Sprite {
                    color: Color::WHITE,
                    ..Default::default()
                },
                transform: Transform::from_xyz(
                    trans.x * width, trans.y * height, zpos::TILE
                ),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
        )).id();
        sprites_index.0[c.into()] = e;
    }

    commands.entity(e_map)
        .insert(sprites_index);

    false.into()
}

fn reveal_sprites_onenter_ingame(
    mut q_sprite: Query<&mut Visibility, With<BaseSprite>>,
) {
    for mut vis in &mut q_sprite {
        *vis = Visibility::Visible;
    }
}
