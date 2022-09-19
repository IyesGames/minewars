use crate::prelude::*;

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
    }
}

fn is_gfx_tilemap_backend_enabled(
    backend: Res<MwMapGfxBackend>,
) -> bool {
    *backend == MwMapGfxBackend::Tilemap
}

#[derive(Component)]
struct DecalTilemap(Entity);
#[derive(Component)]
struct GentsTilemap(Entity);
#[derive(Component)]
struct DigitTilemap(Entity);

fn setup_tilemaps(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    descriptor: Option<Res<MapDescriptor>>,
    settings_colors: Res<PlayerPaletteSettings>,
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
        Topology::Hex => TilemapGridSize { x: 224.0, y: 256.0 },
        Topology::Sq | Topology::Sqr => TilemapGridSize { x: 224.0, y: 224.0 },
    };
    let tmap_texture = match descriptor.topology {
        Topology::Hex => tiles.tiles6[0].clone(),
        Topology::Sq | Topology::Sqr => tiles.tiles4[0].clone(),
    };

    // the tilemaps for upper layers:
    // land decals
    let e_tmap_decal = commands.spawn().id();
    // game entities
    let e_tmap_gents = commands.spawn().id();
    // digits
    let e_tmap_digit = commands.spawn().id();

    // the base tilemap:
    let e_tmap_base = commands.spawn()
        .insert(DecalTilemap(e_tmap_decal))
        .insert(GentsTilemap(e_tmap_gents))
        .insert(DigitTilemap(e_tmap_digit))
        .id();

    // TileStorages
    let mut tstor_base = TileStorage::empty(tmap_size);
    let mut tstor_decal = TileStorage::empty(tmap_size);
    let mut tstor_gents = TileStorage::empty(tmap_size);
    let mut tstor_digit = TileStorage::empty(tmap_size);

    // for x in 0..200 {
    //     for y in 0..200 {
    //         let tile_pos = TilePos { x, y };
    //         let tile_entity = commands
    //             .spawn()
    //             .insert_bundle(TileBundle {
    //                 position: tile_pos,
    //                 tilemap_id: TilemapId(e_tmap_base),
    //                 texture: TileTexture(0),
    //                 ..Default::default()
    //             })
    //             .id();
    //         tstor_base.set(&tile_pos, Some(tile_entity));
    //     }
    // }

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
        });
        tstor_base.set(pos, Some(e));
    }

    // generate water fade-out effect

    commands.entity(e_tmap_base).insert_bundle(TilemapBundle {
        grid_size: TilemapGridSize { x: 224.0, y: 256.0 },
        size: tmap_size,
        storage: tstor_base,
        texture: TilemapTexture(tmap_texture),
        tile_size: TilemapTileSize { x: 256.0, y: 256.0 },
        spacing: TilemapSpacing { x: 0.0, y: 0.0 },
        map_type: tmap_mesh_type,
        transform: Transform::from_xyz(
            - 128.0 * 224.0 * 1.5 - 256.0 * 0.5,
            - 128.0 * 256.0 * 0.75 - 256.0 * 0.5,
            0.0,
        ),
        ..Default::default()
    });

    debug!("Setup grid tiles rendering using Bevy ECS Tilemap!");

    *done = true;

    (*done).into()
}
