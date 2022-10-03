use crate::{prelude::*, camera::translation_tmap};

use super::*;

pub struct MapGfxTilemapPlugin;

impl Plugin for MapGfxTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            setup_tilemaps
                .track_progress()
                .run_in_state(AppGlobalState::GameLoading)
                .run_if(is_gfx_tilemap_backend_enabled)
        );
        app.add_system(base_kind_changed
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_tilemap_backend_enabled)
        );
        app.add_system(tile_owner_color
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_tilemap_backend_enabled)
            .after(MapLabels::TileOwner)
            .after(MapLabels::TileVisible)
        );
        app.add_system(tile_digit_sprite_mgr
            .run_in_state(AppGlobalState::InGame)
            .run_if(is_gfx_tilemap_backend_enabled)
            .after(MapLabels::TileDigit)
        );
    }
}

fn is_gfx_tilemap_backend_enabled(
    backend: Res<MwMapGfxBackend>,
) -> bool {
    *backend == MwMapGfxBackend::Tilemap
}

#[derive(Component)]
struct BaseTilemap;
#[derive(Component)]
struct RoadsTilemap;
#[derive(Component)]
struct GentsTilemap;
#[derive(Component)]
struct DigitTilemap;

fn setup_tilemaps(
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

    if q_tile.is_empty() {
        return false.into();
    }

    let map_z = 0.0;

    let tmap_size = TilemapSize {
        // x: descriptor.size as u32 * 2 + 1,
        // y: descriptor.size as u32 * 2 + 1,
        x: 256,
        y: 256,
    };

    let tmap_mesh_type = match descriptor.topology {
        Topology::Hex => TilemapType::Hexagon(HexCoordSystem::Row),
        Topology::Sq | Topology::Sqr => TilemapType::square(false),
    };
    let tmap_grid_size = match descriptor.topology {
        Topology::Hex => TilemapGridSize { x: zoom.desc.offset6.0 as f32, y: zoom.desc.offset6.1 as f32 },
        Topology::Sq | Topology::Sqr => TilemapGridSize { x: zoom.desc.offset4.0 as f32, y: zoom.desc.offset4.1 as f32 },
    };
    let tmap_tile_size = TilemapTileSize { x: zoom.desc.size as f32, y: zoom.desc.size as f32 };
    let tmap_texture = match descriptor.topology {
        Topology::Hex => tiles.tiles6[zoom.i].clone(),
        Topology::Sq | Topology::Sqr => tiles.tiles4[zoom.i].clone(),
    };
    let roads_texture = match descriptor.topology {
        Topology::Hex => tiles.roads6[zoom.i].clone(),
        Topology::Sq | Topology::Sqr => tiles.roads4[zoom.i].clone(),
    };

    // the tilemaps:
    let e_tmap_base = commands.spawn().insert(BaseTilemap).id();
    let e_tmap_roads = commands.spawn().insert(RoadsTilemap).id();
    let e_tmap_gents = commands.spawn().insert(GentsTilemap).id();
    let e_tmap_digit = commands.spawn().insert(DigitTilemap).id();

    // TileStorages
    let mut tstor_base = TileStorage::empty(tmap_size);
    let tstor_roads = TileStorage::empty(tmap_size);
    let mut tstor_gents = TileStorage::empty(tmap_size);
    let tstor_digit = TileStorage::empty(tmap_size);

    for (e, kind, pos) in q_tile.iter() {
        let i_base = match kind {
            TileKind::Water => tileid::tiles::WATER,
            TileKind::Regular | TileKind::Road => tileid::tiles::LAND,
            TileKind::Mountain => tileid::tiles::MTN,
            TileKind::Fertile => tileid::tiles::FERTILE,
        };
        commands.entity(e).insert_bundle(TileBundle {
            position: *pos,
            texture: TileTexture(i_base as u32),
            tilemap_id: TilemapId(e_tmap_base),
            visible: TileVisible(true),
            color: TileColor(if *kind == TileKind::Water {
                Color::WHITE
            } else {
                settings_colors.visible[0]
            }),
            ..Default::default()
        }).insert(BaseSprite);
        tstor_base.set(pos, Some(e));
    }

    // TODO generate water fade-out effect

    let trans = translation_tmap(descriptor.topology, &zoom.desc);

    commands.entity(e_tmap_base).insert_bundle(TilemapBundle {
        grid_size: tmap_grid_size,
        size: tmap_size,
        storage: tstor_base,
        texture: TilemapTexture(tmap_texture),
        tile_size: tmap_tile_size,
        spacing: TilemapSpacing { x: 0.0, y: 0.0 },
        map_type: tmap_mesh_type,
        transform: Transform::from_translation(
            trans.extend(map_z)
        ),
        ..Default::default()
    });

    commands.entity(e_tmap_roads).insert_bundle(TilemapBundle {
        grid_size: tmap_grid_size,
        size: tmap_size,
        storage: tstor_roads,
        texture: TilemapTexture(roads_texture),
        tile_size: tmap_tile_size,
        spacing: TilemapSpacing { x: 0.0, y: 0.0 },
        map_type: tmap_mesh_type,
        transform: Transform::from_translation(
            trans.extend(map_z + zpos::ROAD)
        ),
        ..Default::default()
    });

    commands.entity(e_tmap_gents).insert_bundle(TilemapBundle {
        grid_size: tmap_grid_size,
        size: tmap_size,
        storage: tstor_gents,
        texture: TilemapTexture(tiles.gents[zoom.i].clone()),
        tile_size: tmap_tile_size,
        spacing: TilemapSpacing { x: 0.0, y: 0.0 },
        map_type: tmap_mesh_type,
        transform: Transform::from_translation(
            trans.extend(map_z + zpos::GENTS)
        ),
        ..Default::default()
    });

    commands.entity(e_tmap_digit).insert_bundle(TilemapBundle {
        grid_size: tmap_grid_size,
        size: tmap_size,
        storage: tstor_digit,
        texture: TilemapTexture(tiles.digits[zoom.i].clone()),
        tile_size: tmap_tile_size,
        spacing: TilemapSpacing { x: 0.0, y: 0.0 },
        map_type: tmap_mesh_type,
        transform: Transform::from_translation(
            trans.extend(map_z + zpos::DIGIT)
        ),
        ..Default::default()
    });

    debug!("Setup grid tiles rendering using Bevy ECS Tilemap!");

    *done = true;

    (*done).into()
}

fn tile_digit_sprite_mgr(
    mut commands: Commands,
    q_tile: Query<
        (Entity, &TilePos, &TileDigit, Option<&TileDigitSprite>),
        (With<BaseSprite>, Changed<TileDigit>)
    >,
    mut q_digit: Query<&mut TileTexture, With<DigitSprite>>,
    mut q_tmap: Query<(Entity, &mut TileStorage), With<DigitTilemap>>,
) {
    if let Ok((e_tmap, mut stor)) = q_tmap.get_single_mut() {
        for (e, pos, digit, spr_digit) in q_tile.iter() {
            if let Some(spr_digit) = spr_digit {
                // there is an existing digit entity we can reuse (or despawn)
                if digit.0 > 0 {
                    let mut tiletex = q_digit.get_mut(spr_digit.0).unwrap();
                    tiletex.0 = digit.0 as u32;
                } else {
                    commands.entity(spr_digit.0).despawn();
                    commands.entity(e).remove::<TileDigitSprite>();
                    stor.set(pos, None);
                }
            } else if digit.0 > 0 {
                dbg!(digit.0);
                // create a new digit entity
                let e_digit = commands.spawn_bundle(TileBundle {
                    position: *pos,
                    tilemap_id: TilemapId(e_tmap),
                    texture: TileTexture(digit.0 as u32),
                    ..Default::default()
                })
                    .insert(MapCleanup)
                    .insert(DigitSprite)
                    .id();
                commands.entity(e).insert(TileDigitSprite(e_digit));
                stor.set(pos, Some(e_digit));
            }
        }
    }
}

fn tile_owner_color(
    settings_colors: Res<PlayerPaletteSettings>,
    mut q_tile: Query<
        (&TileKind, &TileOwner, &TileFoW, &mut TileColor),
        (With<BaseSprite>, Or<(Changed<TileOwner>, Changed<TileFoW>)>)
    >,
) {
    for (kind, owner, tilevis, mut color) in q_tile.iter_mut() {
        if !kind.ownable() {
            continue;
        }

        color.0 = if tilevis.0 {
            settings_colors.visible[owner.0.i()]
        } else {
            settings_colors.fog[owner.0.i()]
        }
    }
}

fn base_kind_changed(
    mut q_tile: Query<
        (&TileKind, &mut TileTexture),
        (With<BaseSprite>, Changed<TileKind>)
    >,
) {
    for (kind, mut sprite) in q_tile.iter_mut() {
        let index = match kind {
            TileKind::Water => {
                tileid::tiles::WATER
            }
            TileKind::Regular => {
                tileid::tiles::LAND
            }
            TileKind::Fertile => {
                tileid::tiles::FERTILE
            }
            TileKind::Mountain => {
                tileid::tiles::MTN
            }
            TileKind::Road => {
                todo!()
            }
        };
        sprite.0 = index;
    }
}
