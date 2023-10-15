use crate::assets::GameAssets;
use crate::prelude::*;

use bevy::render::render_resource::FilterMode;
use mw_app::player::PlayersIndex;
use mw_app::view::PlidViewing;
use mw_app::view::ViewMapData;
use mw_common::grid::*;
use mw_common::game::*;
use mw_app::map::*;
use mw_common::plid::PlayerId;

use super::*;

pub struct Gfx2dTilemapPlugin;

impl Plugin for Gfx2dTilemapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, prealloc_tilemap);
        app.add_systems(OnExit(AppState::AssetsLoading), preprocess_tilemap_assets);
        app.add_systems(Update, (
            (
                setup_tilemap::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                setup_tilemap::<Sq>
                    .in_set(MapTopologySet(Topology::Sq)),
            )
                .in_set(Gfx2dTileSetupSet)
                .run_if(not(resource_exists::<TilemapInitted>())),
            (
                tile_kind::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                tile_kind::<Sq>
                    .in_set(MapTopologySet(Topology::Sq)),
                tile_owner.after(MapUpdateSet::TileOwner),
                digit_tilemap_mgr.after(MapUpdateSet::TileDigit),
                gent_tilemap_mgr.after(MapUpdateSet::TileGent),
                overlay_tilemap_mgr,
                tilemap_reghighlight.run_if(resource_changed::<GridCursorTileEntity>()),
            )
                .run_if(resource_exists::<TilemapInitted>()),
        ).in_set(Gfx2dSet::Tilemap));
    }
}

#[derive(Resource, Clone)]
struct Tilemaps {
    base: Entity,
    digit: Entity,
    road: Entity,
    gent: Entity,
    overlay: Entity,
    reghighlight: Entity,
}

#[derive(Component)]
struct BaseTilemap;
#[derive(Component)]
struct DigitTilemap;
#[derive(Component)]
struct GentTilemap;
#[derive(Component)]
struct RoadTilemap;
#[derive(Component)]
struct OverlayTilemap;
#[derive(Component)]
struct RegHighlightTilemap;

#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileDigitEntity(Entity);
#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileGentEntity(Entity);

/// Reserve the tilemap entities first thing on app startup
/// Workaround for generation>0 bug
fn prealloc_tilemap(
    world: &mut World,
) {
    let base = world.spawn(BaseTilemap).id();
    let gent = world.spawn(GentTilemap).id();
    let digit = world.spawn(DigitTilemap).id();
    let overlay = world.spawn(OverlayTilemap).id();
    let reghighlight = world.spawn(RegHighlightTilemap).id();
    let road = world.spawn(RoadTilemap).id();
    world.insert_resource(Tilemaps {
        base, digit, road, gent, overlay, reghighlight,
    })
}

fn preprocess_tilemap_assets(
    gas: Res<GameAssets>,
    ast: Res<Assets<TextureAtlas>>,
    proc: ResMut<ArrayTextureLoader>,
) {
    let tile_size = TilemapTileSize {
        x: 128.0, y: 128.0,
    };
    proc.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(
            ast.get(&gas.sprites).unwrap().texture.clone(),
        ),
        tile_size,
        filter: Some(FilterMode::Linear),
        ..Default::default()
    });
    proc.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(
            ast.get(&gas.roads6).unwrap().texture.clone(),
        ),
        tile_size,
        filter: Some(FilterMode::Linear),
        ..Default::default()
    });
    proc.add(TilemapArrayTexture {
        texture: TilemapTexture::Single(
            ast.get(&gas.roads4).unwrap().texture.clone(),
        ),
        tile_size,
        filter: Some(FilterMode::Linear),
        ..Default::default()
    });
}

fn setup_tilemap<C: Coord>(
    world: &mut World,
) {
    let (img_sprites, img_roads) = {
        let gas = world.resource::<GameAssets>();
        let ast = world.resource::<Assets<TextureAtlas>>();
        (
            ast.get(&gas.sprites).unwrap().texture.clone(),
            match C::TOPOLOGY {
                Topology::Hex => ast.get(&gas.roads6).unwrap().texture.clone(),
                Topology::Sq => ast.get(&gas.roads4).unwrap().texture.clone(),
            },
        )
    };
    let index = world.remove_resource::<MapTileIndex<C>>().unwrap();

    let map_size = world.resource::<MapDescriptor>().size;
    let tm_size = TilemapSize {
        x: map_size as u32 * 2 + 1,
        y: map_size as u32 * 2 + 1,
    };

    let (sprite_index, width, height, map_type) = match C::TOPOLOGY {
        Topology::Hex => (
            super::sprite::TILES6 + super::sprite::TILE_WATER,
            super::sprite::WIDTH6, super::sprite::HEIGHT6,
            TilemapType::Hexagon(HexCoordSystem::Row),
        ),
        Topology::Sq => (
            super::sprite::TILES4 + super::sprite::TILE_WATER,
            super::sprite::WIDTH4, super::sprite::HEIGHT4,
            TilemapType::Square,
        ),
    };

    let tilemaps = world.resource::<Tilemaps>().clone();

    let mut ts_base = TileStorage::empty(tm_size);

    for (c, &e) in index.0.iter() {
        let tilepos = TilePos {
            x: (c.x() as i32 + map_size as i32) as u32,
            y: (c.y() as i32 + map_size as i32) as u32,
        };
        world.entity_mut(e).insert((
            BaseSprite,
            TileBundle {
                position: tilepos,
                texture_index: TileTextureIndex(sprite_index as u32),
                tilemap_id: TilemapId(tilemaps.base),
                visible: TileVisible(true),
                color: TileColor(Color::WHITE),
                ..Default::default()
            },
        ));
        ts_base.set(&tilepos, e);
    }

    let tile_size = TilemapTileSize {
        x: 128.0, y: 128.0,
    };
    let grid_size = TilemapGridSize {
        x: width as f32,
        y: height as f32,
    };

    world.entity_mut(tilemaps.base).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tm_size,
        storage: ts_base,
        texture: TilemapTexture::Single(img_sprites.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&tm_size, &grid_size, &map_type, zpos::TILE),
        ..Default::default()
    });
    world.entity_mut(tilemaps.gent).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tm_size,
        storage: TileStorage::empty(tm_size),
        texture: TilemapTexture::Single(img_sprites.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&tm_size, &grid_size, &map_type, zpos::GENTS),
        ..Default::default()
    });
    world.entity_mut(tilemaps.digit).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tm_size,
        storage: TileStorage::empty(tm_size),
        texture: TilemapTexture::Single(img_sprites.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&tm_size, &grid_size, &map_type, zpos::DIGIT),
        ..Default::default()
    });
    world.entity_mut(tilemaps.overlay).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tm_size,
        storage: TileStorage::empty(tm_size),
        texture: TilemapTexture::Single(img_sprites.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&tm_size, &grid_size, &map_type, zpos::OVERLAYS),
        ..Default::default()
    });
    world.entity_mut(tilemaps.reghighlight).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tm_size,
        storage: TileStorage::empty(tm_size),
        texture: TilemapTexture::Single(img_sprites.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&tm_size, &grid_size, &map_type, zpos::REGHILIGHT),
        ..Default::default()
    });
    world.entity_mut(tilemaps.road).insert(TilemapBundle {
        grid_size,
        map_type,
        size: tm_size,
        storage: TileStorage::empty(tm_size),
        texture: TilemapTexture::Single(img_roads.clone()),
        tile_size,
        transform: get_tilemap_center_transform(&tm_size, &grid_size, &map_type, zpos::ROAD),
        ..Default::default()
    });

    world.insert_resource(index);

    debug!("Initialized map using Tilemap renderer.");
    world.insert_resource(TilemapInitted);
}

fn tile_kind<C: Coord>(
    mut q: Query<
        (&mut TileTextureIndex, &mut TileColor, &TileKind, &MwTilePos),
        (With<BaseSprite>, Changed<TileKind>)
    >,
    plids: Res<PlayersIndex>,
    viewing: Res<PlidViewing>,
    q_view: Query<&ViewMapData<C>>,
) {
    let i_base = match C::TOPOLOGY {
        Topology::Hex => super::sprite::TILES6,
        Topology::Sq => super::sprite::TILES4,
    };

    for (mut index, mut color, kind, pos) in &mut q {
        index.0 = (i_base + match kind {
            TileKind::Water => super::sprite::TILE_WATER,
            TileKind::Foundation => super::sprite::TILE_FOUNDATION,
            TileKind::Regular => super::sprite::TILE_LAND,
            TileKind::Fertile => super::sprite::TILE_FERTILE,
            TileKind::Forest => super::sprite::TILE_FOREST,
            TileKind::Mountain => super::sprite::TILE_MTN,
            TileKind::Destroyed => super::sprite::TILE_DEAD,
        }) as u32;
        if *kind == TileKind::Water {
            let e_plid = plids.0.get(viewing.0.i())
                .expect("Plid has no entity??");
            let view = q_view.get(*e_plid)
                .expect("Plid has no view??");
            let a = fancytint(
                view.0.size(),
                C::from(pos.0),
                |c| view.0[c].kind()
            );
            color.0.set_a(a);
        } else {
            color.0.set_a(1.0);
        }
    }
}

fn tile_owner(
    settings: Res<AllSettings>,
    mut q: Query<(&mut TileColor, &TileOwner), Changed<TileOwner>>,
) {
    for (mut color, owner) in &mut q {
        color.0 = settings.player_colors.visible[owner.0.i()].into();
        // FIXME: this is to work around bevy_ecs_tilemap broken sRGB
        let rgba = color.0.as_linear_rgba_f32();
        color.0 = Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3]);
    }
}

fn digit_tilemap_mgr(
    mut commands: Commands,
    desc: Res<MapDescriptor>,
    q_tile: Query<
        (Entity, &MwTilePos, &TileDigit, Option<&TileDigitEntity>),
        (With<BaseSprite>, Changed<TileDigit>)
    >,
    mut q_digit: Query<&mut TileTextureIndex, With<DigitSprite>>,
    mut q_tm_digit: Query<(Entity, &mut TileStorage), With<DigitTilemap>>,
) {
    for (e, coord, digit, spr_digit) in q_tile.iter() {
        let tilepos = TilePos {
            x: (coord.0.x() as i32 + desc.size as i32) as u32,
            y: (coord.0.y() as i32 + desc.size as i32) as u32,
        };

        let i_dig = if digit.1 {
            super::sprite::DIGSTAR + digit.0 as usize
        } else {
            super::sprite::DIG + digit.0 as usize
        };

        if let Some(spr_digit) = spr_digit {
            // there is an existing digit entity we can reuse (or despawn)
            if digit.0 > 0 {
                let e_digit = spr_digit.0;
                let mut index = q_digit.get_mut(e_digit).unwrap();
                index.0 = i_dig as u32;
            } else {
                commands.entity(spr_digit.0).despawn_recursive();
                commands.entity(e).remove::<TileDigitEntity>();
                let (_, mut ts_digit) = q_tm_digit.single_mut();
                ts_digit.remove(&tilepos);
            }
        } else if digit.0 > 0 {
            // create a new digit entity
            let (e_tm, mut ts_digit) = q_tm_digit.single_mut();
            let e_digit = commands.spawn((
                DigitSprite,
                MwTilePos(coord.0),
                TileBundle {
                    position: tilepos,
                    texture_index: TileTextureIndex(i_dig as u32),
                    tilemap_id: TilemapId(e_tm),
                    visible: TileVisible(true),
                    color: TileColor(Color::WHITE),
                    ..Default::default()
                },
            )).id();
            commands.entity(e).insert(TileDigitEntity(e_digit));
            ts_digit.set(&tilepos, e_digit);
        }
    }
}

fn gent_tilemap_mgr(
    mut commands: Commands,
    settings: Res<AllSettings>,
    desc: Res<MapDescriptor>,
    viewing: Res<PlidViewing>,
    q_tile: Query<
        (Entity, &MwTilePos, &TileGent, Option<&TileGentEntity>),
        (With<BaseSprite>, Changed<TileGent>)
    >,
    mut q_gent: Query<(&mut TileTextureIndex, &mut TileColor), With<GentSprite>>,
    mut q_tm_gent: Query<(Entity, &mut TileStorage), With<GentTilemap>>,
) {
    for (e, coord, gent, spr_gent) in q_tile.iter() {
        let tilepos = TilePos {
            x: (coord.0.x() as i32 + desc.size as i32) as u32,
            y: (coord.0.y() as i32 + desc.size as i32) as u32,
        };
        let (i_gent, clr_gent) = match gent {
            TileGent::Empty |
            TileGent::Item(ItemKind::Safe) |
            TileGent::Flag(PlayerId::Neutral)=> {
                if let Some(spr_gent) = spr_gent {
                    commands.entity(spr_gent.0).despawn_recursive();
                    commands.entity(e).remove::<TileGentEntity>();
                    let (_, mut ts_gent) = q_tm_gent.single_mut();
                    ts_gent.remove(&tilepos);
                }
                continue;
            }
            TileGent::Flag(plid) => {
                if *plid == viewing.0 {
                    (
                        super::sprite::FLAGS + settings.player_colors.flag_style as usize,
                        Color::WHITE,
                    )
                } else {
                    (
                        super::sprite::FLAGS,
                        settings.player_colors.visible[plid.i()].into(),
                    )
                }
            }
            TileGent::Item(ItemKind::Mine) => (super::sprite::GENT_MINE, Color::WHITE),
            TileGent::Item(ItemKind::Decoy) =>(super::sprite::GENT_DECOY, Color::WHITE),
            TileGent::Item(ItemKind::Flashbang) => (super::sprite::GENT_FLASH, Color::WHITE),
            TileGent::Structure(kind) => {
                (match kind {
                    StructureKind::Barricade => super::sprite::GENT_WALL,
                    StructureKind::WatchTower => super::sprite::GENT_TOWER,
                    StructureKind::Bridge => todo!(),
                    StructureKind::Road => panic!("Roads must use TileRoads, not TileGent"),
                }, Color::WHITE)
            }
            TileGent::Cit(_) => (super::sprite::GENT_CIT, Color::WHITE),
        };
        // FIXME: this is to work around bevy_ecs_tilemap broken sRGB
        let rgba = clr_gent.as_linear_rgba_f32();
        let clr_gent = Color::rgba(rgba[0], rgba[1], rgba[2], rgba[3]);

        if let Some(spr_gent) = spr_gent {
            // there is an existing gent entity we can reuse
            let e_gent = spr_gent.0;
            let (mut index, mut color) = q_gent.get_mut(e_gent).unwrap();
            index.0 = i_gent as u32;
            color.0 = clr_gent;
        } else {
            // create a new gent entity
            let (e_tm, mut ts_gent) = q_tm_gent.single_mut();
            let e_gent = commands.spawn((
                GentSprite,
                MwTilePos(coord.0),
                TileBundle {
                    position: tilepos,
                    texture_index: TileTextureIndex(i_gent as u32),
                    tilemap_id: TilemapId(e_tm),
                    visible: TileVisible(true),
                    color: TileColor(clr_gent),
                    ..Default::default()
                },
            )).id();
            commands.entity(e).insert(TileGentEntity(e_gent));
            ts_gent.set(&tilepos, e_gent);
        }
    }
}

fn overlay_tilemap_mgr(
    mut commands: Commands,
    time: Res<Time>,
    mut q_expl: Query<(
        Entity, &TileExplosion, Option<&mut ExplosionSprite>, Option<&mut TileColor>,
    )>,
    q_tile: Query<&TilePos, With<BaseSprite>>,
    mut q_tm_overlay: Query<(Entity, &mut TileStorage), With<OverlayTilemap>>,
) {
    for (e, expl, spr_expl, sprite) in &mut q_expl {
        if let (Some(mut spr_expl), opt_color) = (spr_expl, sprite) {
            // sprite already set up, manage it
            spr_expl.timer.tick(time.delta());
            if spr_expl.timer.finished() {
                commands.entity(e).despawn_recursive();
                let tilepos = q_tile.get(expl.0).unwrap();
                let (_, mut ts_overlay) = q_tm_overlay.single_mut();
                ts_overlay.remove(&tilepos);
            }
            if let Some(mut color) = opt_color {
                color.0.set_a(spr_expl.timer.percent_left());
            }
        } else {
            // we have an entity with no sprite, set up the sprite
            let (e_tm, mut ts_overlay) = q_tm_overlay.single_mut();
            let tilepos = q_tile.get(expl.0).unwrap();
            let i_expl = match expl.1 {
                TileExplosionKind::Normal => super::sprite::EXPLOSION_MINE,
                TileExplosionKind::Decoy => super::sprite::EXPLOSION_DECOY,
            };
            commands.entity(e).insert((
                ExplosionSprite {
                    timer: Timer::new(Duration::from_millis(1000), TimerMode::Once),
                },
                TileBundle {
                    position: *tilepos,
                    texture_index: TileTextureIndex(i_expl as u32),
                    tilemap_id: TilemapId(e_tm),
                    visible: TileVisible(true),
                    color: TileColor(Color::WHITE),
                    ..Default::default()
                },
            ));
            ts_overlay.set(&tilepos, e);
        }
    }
}

fn tilemap_reghighlight(
    mut commands: Commands,
    mapdesc: Res<MapDescriptor>,
    cursor_tile: Res<GridCursorTileEntity>,
    q_highlight: Query<Entity, With<RegHighlightSprite>>,
    q_tile: Query<(&TilePos, &TileRegion), With<BaseSprite>>,
    mut q_tm: Query<(Entity, &TilemapSize, &mut TileStorage), With<RegHighlightTilemap>>,
    mut last_region: Local<Option<u8>>,
) {
    if let Some(e_tile) = cursor_tile.0 {
        let Ok(region) = q_tile.get(e_tile) else {
            return;
        };
        let region = region.1.0;
        if *last_region != Some(region) {
            *last_region = Some(region);
            // clear old
            if !q_highlight.is_empty() {
                let (_, tm_size, mut ts) = q_tm.single_mut();
                *ts = TileStorage::empty(*tm_size);
                for e in &q_highlight {
                    commands.entity(e).despawn_recursive();
                }
            }
            // create new
            let i = match mapdesc.topology {
                Topology::Hex => super::sprite::TILES6 + super::sprite::TILE_HIGHLIGHT,
                Topology::Sq => super::sprite::TILES4 + super::sprite::TILE_HIGHLIGHT,
            };
            let (e_tm, _, mut ts) = q_tm.single_mut();
            for (tilepos, tile_region) in &q_tile {
                if tile_region.0 == region {
                    let e_tile = commands.spawn((
                        RegHighlightSprite,
                        TileBundle {
                            position: *tilepos,
                            texture_index: TileTextureIndex(i as u32),
                            tilemap_id: TilemapId(e_tm),
                            visible: TileVisible(true),
                            color: TileColor(Color::WHITE.with_a(0.25)),
                            ..Default::default()
                        }
                    )).id();
                    ts.set(&tilepos, e_tile);
                }
            }
        }
    } else {
        if !q_highlight.is_empty() {
            let (_, tm_size, mut ts) = q_tm.single_mut();
            *ts = TileStorage::empty(*tm_size);
            for e in &q_highlight {
                commands.entity(e).despawn_recursive();
            }
        }
    }
}
