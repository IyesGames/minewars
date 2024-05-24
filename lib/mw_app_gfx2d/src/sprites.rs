use mw_app_core::{map::{tile::{coalesce_tile_update_events, MwMapTile, MwTilePos, TileUpdateEvent, TileUpdateSS}, *}, session::{PlayersIndex, PlidViewing, SessionGovernor}};
use mw_common::grid::*;

use crate::{assets::Gfx2dAssets, misc::*, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        setup_tile_entities
            .track_progress()
            .in_set(Gfx2dImplSet::Sprites)
            .in_set(NeedsMapGovernorSet),
    )
        .in_set(InStateSet(AppState::GameLoading)),
    );
    app.add_systems(OnEnter(AppState::InGame), (
        reveal_sprites_onenter_ingame,
    ));
    app.add_systems(Update, (
        update_sprite_tile_kind,
    )
        .in_set(SetStage::WantChanged(TileUpdateSS))
        .in_set(Gfx2dImplSet::Sprites)
        .in_set(NeedsMapGovernorSet)
    );
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
            Topology::Hex => {
                let c = Hex::from(c);
                if c.ring() > desc.size {
                    continue;
                }
                c.translation()
            },
            Topology::Sq => {
                let c = Sq::from(c);
                if c.ring() > desc.size {
                    continue;
                }
                c.translation()
            },
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

    debug!("Initialized map graphics using 2D Sprites.");

    false.into()
}

fn reveal_sprites_onenter_ingame(
    mut q_sprite: Query<&mut Visibility, With<BaseSprite>>,
) {
    for mut vis in &mut q_sprite {
        *vis = Visibility::Visible;
    }
}

fn update_sprite_tile_kind(
    q_map: Query<(&MapDescriptor, &MapSpritesIndex), With<MapGovernor>>,
    q_tile: Query<(&MwTilePos, &TileKind), With<MwMapTile>>,
    mut q_sprite: Query<&mut TextureAtlas, With<BaseSprite>>,
    mut evr: EventReader<TileUpdateEvent>,
    mut coalesced_updates: Local<HashSet<Entity>>,
) {
    coalesce_tile_update_events(&mut coalesced_updates, &mut evr);
    let (desc, sprindex) = q_map.single();
    let i_base = match desc.topology {
        Topology::Hex => sprite::TILES6,
        Topology::Sq => sprite::TILES4,
    };
    let mut do_update = |e, kind| {
        if let Ok(mut atlas) = q_sprite.get_mut(e) {
            atlas.index = i_base + match kind {
                TileKind::Water => sprite::TILE_WATER,
                TileKind::FoundationRoad => sprite::TILE_FOUNDATION,
                TileKind::FoundationStruct => sprite::TILE_FOUNDATION,
                TileKind::Regular => sprite::TILE_LAND,
                TileKind::Fertile => sprite::TILE_FERTILE,
                TileKind::Forest => sprite::TILE_FOREST,
                TileKind::Mountain => sprite::TILE_MTN,
                TileKind::Destroyed => sprite::TILE_DEAD,
            };
        }
    };
    for e in coalesced_updates.drain() {
        if e == Entity::PLACEHOLDER {
            for (pos, kind) in &q_tile {
                let e_spr = sprindex.0[pos.0];
                do_update(e_spr, *kind);
            }
            break;
        } else {
            if let Ok((pos, kind)) = q_tile.get(e) {
                let e_spr = sprindex.0[pos.0];
                do_update(e_spr, *kind);
            }
        }
    }
}
