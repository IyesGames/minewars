use mw_common::game::{MapDescriptor, TileKind};
use mw_common::grid::*;

use crate::prelude::*;

use crate::assets::TileAssets;
use crate::camera::GridCursor;
use crate::settings::PlayerPaletteSettings;

use super::{MapCleanup, TileCoord, MapLabels, TileOwner, TileVisible, TileDigit};
use super::tileid::{self, CoordTileids};

#[derive(Component)]
struct CursorSprite;
#[derive(Component)]
struct BaseSprite;
#[derive(Component)]
struct DecalSprite;
#[derive(Component)]
struct DigitSprite;

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

pub struct MapGfxSpritesPlugin;

impl Plugin for MapGfxSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppGlobalState::InGame, setup_cursor);
        app.add_system(
            setup_tiles
                .track_progress()
                .run_in_state(AppGlobalState::GameLoading)
        );
        app.add_system(
            cursor_sprite
                .run_in_state(AppGlobalState::InGame)
                .after("cursor")
        );
        app.add_system_set(ConditionSet::new()
            .run_in_state(AppGlobalState::InGame)
            .with_system(tile_decal_sprite_mgr)
            .with_system(cursor_sprite)
            .into()
        );
        app.add_system(tile_owner_color
            .run_in_state(AppGlobalState::InGame)
            .after(MapLabels::TileOwner)
            .after(MapLabels::TileVisible)
        );
        app.add_system(tile_digit_sprite_mgr
            .run_in_state(AppGlobalState::InGame)
            .after(MapLabels::TileDigit)
            .after(MapLabels::TileVisible)
        );
    }
}

fn setup_tiles(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    descriptor: Option<Res<MapDescriptor>>,
    settings_colors: Res<PlayerPaletteSettings>,
    q_tile: Query<(Entity, &TileKind, &TileCoord)>,
    mut done: Local<bool>,
) -> Progress {
    let descriptor = if let Some(descriptor) = descriptor {
        // reset for new game
        if descriptor.is_changed() {
            *done = false;
        }

        descriptor
    } else {
        return false.into();
    };

    if *done {
        return true.into();
    }

    let mut done_now = false;
    for (e, kind, pos) in q_tile.iter() {
        let index = match (descriptor.topology, kind) {
            (_, TileKind::Water) => tileid::GEO_WATER,
            (Topology::Hex, _) => Hex::TILEID_LAND,
            (Topology::Sq | Topology::Sqr, _) => Sq::TILEID_LAND,
        };
        let xy = translation_pos(descriptor.topology, pos.0);
        commands.entity(e).insert_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index,
                color: settings_colors.visible[0],
                ..Default::default()
            },
            texture_atlas: tiles.atlas.clone(),
            transform: Transform::from_translation(xy.extend(0.0)),
            ..Default::default()
        }).insert(BaseSprite);
        *done = true;
        done_now = true;
    }

    if done_now {
        debug!("Setup grid tiles rendering using Bevy Sprites!");
    }

    (*done).into()
}

fn tile_decal_sprite_mgr(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    q_tile: Query<
        (Entity, &TileCoord, &TileKind, &Transform, Option<&TileDecalSprite>),
        Changed<TileKind>
    >,
) {
    for (e, coord, kind, xf, spr_decal) in q_tile.iter() {
        let mut xyz = xf.translation;
        // UGLY: maybe don't hardcode this here?
        xyz.z += 1.0;

        // remove the old decal
        if let Some(spr_decal) = spr_decal {
            commands.entity(spr_decal.0).despawn();
            commands.entity(e).remove::<TileDecalSprite>();
        }

        let index = match kind {
            TileKind::Water | TileKind::Regular => {
                continue;
            }
            TileKind::Fertile => {
                tileid::GEO_FERTILE
            }
            TileKind::Mountain => {
                tileid::GEO_MOUNTAIN
            }
            TileKind::Road => {
                todo!()
            }
        };

        let e_decal = commands.spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite {
                index,
                ..Default::default()
            },
            texture_atlas: tiles.atlas.clone(),
            transform: xf.clone(),
            ..Default::default()
        })
            .insert(MapCleanup)
            .insert(DecalSprite)
            .insert(coord.clone())
            .id();
        commands.entity(e).insert(TileDecalSprite(e_decal));
    }
}

fn tile_digit_sprite_mgr(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    q_tile: Query<
        (Entity, &TileCoord, &TileVisible, &TileDigit, &Transform, Option<&TileDigitSprite>),
        (With<BaseSprite>, Or<(Changed<TileDigit>, Changed<TileVisible>)>)
    >,
    mut q_digit: Query<&mut TextureAtlasSprite, With<DigitSprite>>,
) {
    for (e, coord, tilevis, digit, xf, spr_digit) in q_tile.iter() {
        // we aren't supposed to show digits in fog of war;
        if !tilevis.0 {
            // remove any if present
            if let Some(spr_digit) = spr_digit {
                commands.entity(spr_digit.0).despawn();
                commands.entity(e).remove::<TileDigitSprite>();
            }
            continue;
        }

        let mut xyz = xf.translation;
        // UGLY: maybe don't hardcode this here?
        xyz.z += 2.0;

        if let Some(spr_digit) = spr_digit {
            // there is an existing digit entity we can reuse (or despawn)
            if digit.0 > 0 {
                let e_digit = spr_digit.0;
                let mut sprite = q_digit.get_mut(e_digit).unwrap();
                sprite.index = tileid::DIGITS[digit.0 as usize];
            } else {
                commands.entity(spr_digit.0).despawn();
                commands.entity(e).remove::<TileDigitSprite>();
            }
        } else if digit.0 > 0 {
            // create a new digit entity
            let e_digit = commands.spawn_bundle(SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: tileid::DIGITS[digit.0 as usize],
                    ..Default::default()
                },
                texture_atlas: tiles.atlas.clone(),
                transform: xf.clone(),
                ..Default::default()
            })
                .insert(MapCleanup)
                .insert(DigitSprite)
                .insert(coord.clone())
                .id();
            commands.entity(e).insert(TileDigitSprite(e_digit));
        }
    }
}

fn setup_cursor(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    descriptor: Res<MapDescriptor>,
) {
    let index = match descriptor.topology {
        Topology::Hex => Hex::TILEID_CURSOR,
        Topology::Sq => Sq::TILEID_CURSOR,
        Topology::Sqr => Sq::TILEID_CURSOR,
    };

    commands.spawn_bundle(SpriteSheetBundle {
        sprite: TextureAtlasSprite {
            index,
            ..Default::default()
        },
        texture_atlas: tiles.atlas.clone(),
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..Default::default()
    })
        .insert(CursorSprite)
        .insert(MapCleanup);
}

fn tile_owner_color(
    settings_colors: Res<PlayerPaletteSettings>,
    mut q_tile: Query<
        (&TileKind, &TileOwner, &TileVisible, &mut TextureAtlasSprite),
        (With<BaseSprite>, Or<(Changed<TileOwner>, Changed<TileVisible>)>)
    >,
) {
    for (kind, owner, tilevis, mut sprite) in q_tile.iter_mut() {
        if !kind.ownable() {
            continue;
        }

        sprite.color = if tilevis.0 {
            settings_colors.visible[owner.0.i()]
        } else {
            settings_colors.fog[owner.0.i()]
        }
    }
}

fn cursor_sprite(
    mut q: Query<&mut Transform, With<CursorSprite>>,
    crs: Res<GridCursor>,
    descriptor: Res<MapDescriptor>,
) {
    let mut xf = q.single_mut();
    xf.translation = translation_pos(descriptor.topology, crs.0).extend(10.0);
}

fn translation_c<C: CoordTileids>(c: C) -> Vec2 {
    c.translation() * C::TILE_OFFSET
}

fn translation_pos(topology: Topology, pos: Pos) -> Vec2 {
    match topology {
        Topology::Hex => translation_c(Hex(pos.0, pos.1)),
        Topology::Sq => translation_c(Sq(pos.0, pos.1)),
        _ => unimplemented!(),
    }
}
