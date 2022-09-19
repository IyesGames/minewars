use crate::{prelude::*, camera::ZoomLevel};

use super::{*, tileid::get_rect};

pub struct MapGfxSpritesPlugin;

impl Plugin for MapGfxSpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            setup_tiles
                .track_progress()
                .run_in_state(AppGlobalState::GameLoading)
                .run_if(is_gfx_sprites_backend_enabled)
        );
        app.add_system(explosion_sprite_mgr
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_sprites_backend_enabled)
            .label(MapLabels::ApplyEvents)
        );
        app.add_system_set(ConditionSet::new()
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_sprites_backend_enabled)
            .with_system(tile_decal_sprite_mgr)
            .with_system(mine_active_animation)
            .with_system(explosion_animation)
            .into()
        );
        app.add_system(tile_owner_color
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_sprites_backend_enabled)
            .after(MapLabels::TileOwner)
            .after(MapLabels::TileVisible)
        );
        app.add_system(tile_digit_sprite_mgr
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_sprites_backend_enabled)
            .after(MapLabels::TileDigit)
        );
        app.add_system(mine_sprite_mgr
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_sprites_backend_enabled)
            .after(MapLabels::TileMine)
        );
    }
}

fn is_gfx_sprites_backend_enabled(
    backend: Res<MwMapGfxBackend>,
) -> bool {
    *backend == MwMapGfxBackend::Sprites
}

fn setup_tiles(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    descriptor: Option<Res<MapDescriptor>>,
    settings_colors: Res<PlayerPaletteSettings>,
    zoom: Res<ZoomLevel>,
    q_tile: Query<(Entity, &TileKind, &TilePos)>,
    q_cit: Query<(Entity, &TilePos), With<CitEntity>>,
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

    let map_z = 0.0;

    let mut done_now = false;
    let tmap_texture = match descriptor.topology {
        Topology::Hex => tiles.tiles6[zoom.i].clone(),
        Topology::Sq | Topology::Sqr => tiles.tiles4[zoom.i].clone(),
    };
    for (e, kind, pos) in q_tile.iter() {
        let i_base = match kind {
            TileKind::Water => tileid::tiles::WATER,
            TileKind::Regular | TileKind::Road => tileid::tiles::LAND,
            TileKind::Mountain => tileid::tiles::MTN,
            TileKind::Fertile => tileid::tiles::FERTILE,
        };
        let xy = translation_pos(descriptor.topology, pos.into(), &zoom.desc);
        commands.entity(e).insert_bundle(SpriteBundle {
            sprite: Sprite {
                rect: Some(get_rect(zoom.desc.size, i_base)),
                color: settings_colors.visible[0],
                ..Default::default()
            },
            texture: tmap_texture.clone(),
            transform: Transform::from_translation(xy.extend(map_z)),
            ..Default::default()
        }).insert(BaseSprite);
        *done = true;
        done_now = true;
    }

    // ASSUMES if tiles are ready cits are also ready (setup at the same time)
    for (e, pos) in q_cit.iter() {
        let xy = translation_pos(descriptor.topology, pos.into(), &zoom.desc);
        commands.entity(e).insert_bundle(SpriteBundle {
            sprite: Sprite {
                rect: Some(tileid::get_rect(zoom.desc.size, tileid::gents::CIT)),
                ..Default::default()
            },
            texture: tiles.gents[zoom.i].clone(),
            transform: Transform::from_translation(xy.extend(map_z + zpos::GENTS)),
            ..Default::default()
        }).insert(CitSprite);
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
        (Entity, &TilePos, &TileKind, &Transform, Option<&TileDecalSprite>),
        (With<BaseSprite>, Changed<TileKind>)
    >,
) {
    /*
    for (e, coord, kind, xf, spr_decal) in q_tile.iter() {
        let mut xyz = xf.translation;
        xyz.z += zpos::DECAL;

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
    */
}

fn tile_digit_sprite_mgr(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    zoom: Res<ZoomLevel>,
    q_tile: Query<
        (Entity, &TilePos, &TileDigit, &Transform, Option<&TileDigitSprite>),
        (With<BaseSprite>, Changed<TileDigit>)
    >,
    mut q_digit: Query<&mut Sprite, With<DigitSprite>>,
) {
    for (e, coord, digit, xf, spr_digit) in q_tile.iter() {
        let mut xyz = xf.translation;
        xyz.z += zpos::DIGIT;

        if let Some(spr_digit) = spr_digit {
            // there is an existing digit entity we can reuse (or despawn)
            if digit.0 > 0 {
                let e_digit = spr_digit.0;
                let mut sprite = q_digit.get_mut(e_digit).unwrap();
                sprite.rect = Some(tileid::get_rect(zoom.desc.size, digit.0 as u32));
            } else {
                commands.entity(spr_digit.0).despawn();
                commands.entity(e).remove::<TileDigitSprite>();
            }
        } else if digit.0 > 0 {
            // create a new digit entity
            let e_digit = commands.spawn_bundle(SpriteBundle {
                sprite: Sprite {
                    rect: Some(dbg!(tileid::get_rect(zoom.desc.size, digit.0 as u32))),
                    ..Default::default()
                },
                texture: tiles.digits[zoom.i].clone(),
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
    zoom: Res<ZoomLevel>,
    q_tile: Query<
        (Entity, &TilePos, &TileMine, &Transform, Option<&TileMineSprite>),
        (With<BaseSprite>, Changed<TileMine>)
    >,
    mut q_mine: Query<&mut Sprite, With<MineSprite>>,
) {
    for (e, coord, mine, xf, spr_mine) in q_tile.iter() {
        let mut xyz = xf.translation;
        xyz.z += zpos::GENTS;

        if let Some(display) = mine.0 {
            let index = match display {
                MineDisplayState::Normal(MineKind::Mine) |
                MineDisplayState::Pending(MineKind::Mine) => tileid::gents::MINE,
                MineDisplayState::Normal(MineKind::Decoy) |
                MineDisplayState::Pending(MineKind::Decoy) => tileid::gents::DECOY,
                MineDisplayState::Active => tileid::gents::MINE_ACTIVE,
            };
            let mut color = Color::WHITE;
            if let MineDisplayState::Pending(_) = display {
                color.set_a(0.5);
            }
            let e_mine = if let Some(spr_mine) = spr_mine {
                // reuse existing entity
                let e_mine = spr_mine.0;
                let mut sprite = q_mine.get_mut(e_mine).unwrap();
                sprite.rect = Some(tileid::get_rect(zoom.desc.size, index));
                sprite.color = color;
                e_mine
            } else {
                // spawn new entity
                let e_mine = commands.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        rect: Some(tileid::get_rect(zoom.desc.size, index)),
                        color,
                        ..Default::default()
                    },
                    texture: tiles.gents[zoom.i].clone(),
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
    zoom: Res<ZoomLevel>,
    q_tile: Query<(&Transform, &TilePos), With<BaseSprite>>,
) {
    for ev in evr_map.iter() {
        if ev.plid != my_plid.0 {
            continue;
        }
        if let MapEventKind::Explosion { kind } = ev.kind {
            let e_tile = index.0[ev.c];
            if let Ok((xf, coord)) = q_tile.get(e_tile) {
                let mut xyz = xf.translation;
                xyz.z += zpos::EXPLOSION;
                let index = match kind {
                    MineKind::Mine => tileid::gents::EXPLODE_MINE,
                    MineKind::Decoy => tileid::gents::EXPLODE_DECOY,
                };
                commands.spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        rect: Some(tileid::get_rect(zoom.desc.size, index)),
                        ..Default::default()
                    },
                    texture: tiles.gents[zoom.i].clone(),
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
    mut q: Query<(Entity, &mut Sprite, &mut ExplosionSprite)>,
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
    zoom: Res<ZoomLevel>,
    mut q: Query<(&mut Sprite, &mut MineActiveAnimation)>,
) {
    for (mut sprite, mut anim) in q.iter_mut() {
        anim.timer.tick(time.delta());
        if anim.timer.just_finished() {
            let rect_active = get_rect(zoom.desc.size, tileid::gents::MINE_ACTIVE);
            let rect_inactive = get_rect(zoom.desc.size, tileid::gents::MINE);
            sprite.rect = if let Some(rect_active) = sprite.rect {
                Some(rect_inactive)
            } else {
                Some(rect_active)
            };
        }
    }
}

fn tile_owner_color(
    settings_colors: Res<PlayerPaletteSettings>,
    mut q_tile: Query<
        (&TileKind, &TileOwner, &TileFoW, &mut Sprite),
        (With<BaseSprite>, Or<(Changed<TileOwner>, Changed<TileFoW>)>)
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
