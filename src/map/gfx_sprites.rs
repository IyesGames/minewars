use mw_common::game::{MapDescriptor, TileKind, MineKind};
use mw_common::grid::*;

use crate::prelude::*;

use crate::assets::TileAssets;
use crate::camera::GridCursor;
use crate::settings::PlayerPaletteSettings;

use super::*;
use super::tileid::CoordTileids;

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
            .with_system(mine_active_animation)
            .with_system(explosion_animation)
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
        );
        app.add_system(mine_sprite_mgr
            .run_in_state(AppGlobalState::InGame)
            .after(MapLabels::TileMine)
        );
        app.add_system(explosion_sprite_mgr
            .run_in_state(AppGlobalState::InGame)
            .label(MapLabels::ApplyEvents)
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
        (With<BaseSprite>, Changed<TileKind>)
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
            transform: Transform::from_translation(xyz),
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
        (Entity, &TileCoord, &TileDigit, &Transform, Option<&TileDigitSprite>),
        (With<BaseSprite>, Changed<TileDigit>)
    >,
    mut q_digit: Query<&mut TextureAtlasSprite, With<DigitSprite>>,
) {
    for (e, coord, digit, xf, spr_digit) in q_tile.iter() {
        let mut xyz = xf.translation;
        // UGLY: maybe don't hardcode this here?
        xyz.z += 3.0;

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
                transform: Transform::from_translation(xyz),
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

fn mine_sprite_mgr(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    q_tile: Query<
        (Entity, &TileCoord, &TileMine, &Transform, Option<&TileMineSprite>),
        (With<BaseSprite>, Changed<TileMine>)
    >,
    mut q_mine: Query<&mut TextureAtlasSprite, With<MineSprite>>,
) {
    for (e, coord, mine, xf, spr_mine) in q_tile.iter() {
        let mut xyz = xf.translation;
        // UGLY: maybe don't hardcode this here?
        xyz.z += 2.0;

        if let Some(display) = mine.0 {
            let index = match display {
                MineDisplayState::Normal(MineKind::Mine) |
                MineDisplayState::Pending(MineKind::Mine) => tileid::ITEM_MINE,
                MineDisplayState::Normal(MineKind::Decoy) |
                MineDisplayState::Pending(MineKind::Decoy) => tileid::ITEM_DECOY,
                MineDisplayState::Active => tileid::MINE_ACTIVE,
            };
            let mut color = Color::WHITE;
            if let MineDisplayState::Pending(_) = display {
                color.set_a(0.5);
            }
            let e_mine = if let Some(spr_mine) = spr_mine {
                // reuse existing entity
                let e_mine = spr_mine.0;
                let mut sprite = q_mine.get_mut(e_mine).unwrap();
                sprite.index = index;
                sprite.color = color;
                e_mine
            } else {
                // spawn new entity
                let e_mine = commands.spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index,
                        color,
                        ..Default::default()
                    },
                    texture_atlas: tiles.atlas.clone(),
                    transform: Transform::from_translation(xyz),
                    ..Default::default()
                })
                    .insert(MapCleanup)
                    .insert(MineSprite)
                    .insert(coord.clone())
                    .id();
                commands.entity(e).insert(TileMineSprite(e_mine));
                e_mine
            };
            if display == MineDisplayState::Active {
                commands.entity(e_mine).insert(MineActiveAnimation {
                    timer: Timer::new(Duration::from_millis(125), true),
                });
            } else {
                commands.entity(e_mine).remove::<MineActiveAnimation>();
            }
        } else if let Some(spr_mine) = spr_mine {
            commands.entity(spr_mine.0).despawn();
            commands.entity(e).remove::<TileMineSprite>();
        }
    }
}

fn explosion_sprite_mgr(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    mut evr_map: EventReader<MapEvent>,
    index: Res<TileEntityIndex>,
    my_plid: Res<ActivePlid>,
    q_tile: Query<(&Transform, &TileCoord), With<BaseSprite>>,
) {
    for ev in evr_map.iter() {
        if ev.plid != my_plid.0 {
            continue;
        }
        if let MapEventKind::Explosion { kind } = ev.kind {
            let e_tile = index.0[ev.c];
            if let Ok((xf, coord)) = q_tile.get(e_tile) {
                let mut xyz = xf.translation;
                // UGLY: maybe don't hardcode this here?
                xyz.z += 5.0;
                let index = match kind {
                    MineKind::Mine => tileid::EXPLODE_MINE,
                    MineKind::Decoy => tileid::EXPLODE_DECOY,
                };
                commands.spawn_bundle(SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index,
                        ..Default::default()
                    },
                    texture_atlas: tiles.atlas.clone(),
                    transform: Transform::from_translation(xyz),
                    ..Default::default()
                }).insert(ExplosionSprite {
                    // TODO: make duration configurable via user setting?
                    timer: Timer::new(Duration::from_millis(1250), false),
                })
                    .insert(MapCleanup)
                    .insert(coord.clone());
            }
        }
    }
}

fn explosion_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut q: Query<(Entity, &mut TextureAtlasSprite, &mut ExplosionSprite)>,
) {
    for (e, mut sprite, mut anim) in q.iter_mut() {
        anim.timer.tick(time.delta());
        sprite.color.set_a(anim.timer.percent_left());
        if anim.timer.finished() {
            commands.entity(e).despawn();
        }
    }
}

fn mine_active_animation(
    time: Res<Time>,
    mut q: Query<(&mut TextureAtlasSprite, &mut MineActiveAnimation)>,
) {
    for (mut sprite, mut anim) in q.iter_mut() {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            sprite.index = if sprite.index == tileid::MINE_ACTIVE {
                tileid::ITEM_MINE
            } else {
                tileid::MINE_ACTIVE
            };
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
