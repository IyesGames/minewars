use crate::assets::GameAssets;
use crate::prelude::*;

use mw_app::camera::GridCursor;
use mw_app::camera::GridCursorChangedSet;
use mw_app::player::PlayersIndex;
use mw_app::view::PlidViewing;
use mw_app::view::ViewMapData;
use mw_common::grid::*;
use mw_common::game::*;
use mw_app::map::*;
use mw_common::plid::PlayerId;

use super::*;

pub struct Gfx2dSpritesPlugin;

impl Plugin for Gfx2dSpritesPlugin {
    fn build(&self, app: &mut App) {
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
                tile_digit_sprite_mgr.after(MapUpdateSet::TileDigit),
                tile_gent_sprite_mgr.after(MapUpdateSet::TileGent),
                explosion_sprite_mgr,
                sprites_reghighlight,
            )
                .run_if(resource_exists::<TilemapInitted>()),
        ).in_set(Gfx2dSet::Sprites));
        app.add_systems(OnEnter(AppState::InGame), (
            setup_cursor,
        ).in_set(Gfx2dSet::Any));
        app.add_systems(Update, (
            (
                cursor_sprite::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                cursor_sprite::<Sq>
                    .in_set(MapTopologySet(Topology::Sq)),
            )
                .in_set(GridCursorChangedSet),
        ).in_set(Gfx2dSet::Any));
    }
}

#[derive(Bundle)]
struct CursorSpriteBundle {
    sprite: SpriteSheetBundle,
    pos: MwTilePos,
    marker: CursorSprite,
}

#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileDigitEntity(Entity);
#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileGentEntity(Entity);

fn setup_cursor(
    mut commands: Commands,
    gass: Res<GameAssets>,
    mapdesc: Res<MapDescriptor>,
) {
    let i = match mapdesc.topology {
        Topology::Hex => super::sprite::TILES6 + super::sprite::TILE_CURSOR,
        Topology::Sq => super::sprite::TILES4 + super::sprite::TILE_CURSOR,
    };
    commands.spawn((
        CursorSpriteBundle {
            sprite: SpriteSheetBundle {
                sprite: TextureAtlasSprite {
                    index: i,
                    ..Default::default()
                },
                texture_atlas: gass.sprites.clone(),
                transform: Transform::from_xyz(0.0, 0.0, super::zpos::CURSOR),
                ..Default::default()
            },
            pos: MwTilePos(Pos::origin()),
            marker: CursorSprite,
        },
    ));
}

fn cursor_sprite<C: Coord>(
    mut q: Query<(&mut Transform, &mut MwTilePos), With<CursorSprite>>,
    crs: Res<GridCursor>,
) {
    let Ok((mut xf, mut pos)) = q.get_single_mut() else {
        return;
    };
    *pos = MwTilePos(crs.0);

    let (width, height) = match C::TOPOLOGY {
        Topology::Hex => (
            super::sprite::WIDTH6, super::sprite::HEIGHT6,
        ),
        Topology::Sq => (
            super::sprite::WIDTH4, super::sprite::HEIGHT4,
        ),
    };
    let trans = C::from(crs.0).translation();
    xf.translation = Vec3::new(trans.x * width, trans.y * height, super::zpos::CURSOR);
}

fn setup_tilemap<C: Coord>(
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
        world.entity_mut(e).insert((
            BaseSprite,
            SpriteSheetBundle {
                texture_atlas: texture_atlas.clone(),
                sprite: TextureAtlasSprite {
                    color: Color::WHITE,
                    index: sprite_index,
                    ..Default::default()
                },
                transform: Transform::from_xyz(trans.x * width, trans.y * height, super::zpos::TILE),
                ..Default::default()
            },
        ));
    }
    world.insert_resource(index);
    world.insert_resource(TilemapInitted);
    debug!("Initialized map using Sprites renderer.");
}

fn tile_kind<C: Coord>(
    mut q: Query<
        (&mut TextureAtlasSprite, &TileKind, &MwTilePos),
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

    for (mut spr, kind, pos) in &mut q {
        spr.index = i_base + match kind {
            TileKind::Water => super::sprite::TILE_WATER,
            TileKind::Foundation => super::sprite::TILE_FOUNDATION,
            TileKind::Regular => super::sprite::TILE_LAND,
            TileKind::Fertile => super::sprite::TILE_FERTILE,
            TileKind::Forest => super::sprite::TILE_FOREST,
            TileKind::Mountain => super::sprite::TILE_MTN,
            TileKind::Destroyed => super::sprite::TILE_DEAD,
        };
        if *kind == TileKind::Water {
            let e_plid = plids.0.get(viewing.0.i())
                .expect("Plid has no entity??");
            let view = q_view.get(*e_plid)
                .expect("Plid has no view??");
            let a =  fancytint(
                view.0.size(),
                C::from(pos.0),
                |c| view.0[c].kind()
            );
            spr.color.set_a(a);
        } else {
            spr.color.set_a(1.0);
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

fn tile_digit_sprite_mgr(
    mut commands: Commands,
    assets: Res<GameAssets>,
    q_tile: Query<
        (Entity, &MwTilePos, &TileDigit, &Transform, Option<&TileDigitEntity>),
        (With<BaseSprite>, Changed<TileDigit>)
    >,
    mut q_digit: Query<&mut TextureAtlasSprite, With<DigitSprite>>,
) {
    for (e, coord, digit, xf, spr_digit) in q_tile.iter() {
        let mut trans = xf.translation;
        trans.z = zpos::DIGIT;

        let i_dig = if digit.1 {
            super::sprite::DIGSTAR + digit.0 as usize
        } else {
            super::sprite::DIG + digit.0 as usize
        };

        if let Some(spr_digit) = spr_digit {
            // there is an existing digit entity we can reuse (or despawn)
            if digit.0 > 0 {
                let e_digit = spr_digit.0;
                let mut sprite = q_digit.get_mut(e_digit).unwrap();
                sprite.index = i_dig;
            } else {
                commands.entity(spr_digit.0).despawn();
                commands.entity(e).remove::<TileDigitEntity>();
            }
        } else if digit.0 > 0 {
            // create a new digit entity
            let e_digit = commands.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: i_dig,
                        ..Default::default()
                    },
                    texture_atlas: assets.sprites.clone(),
                    transform: Transform::from_translation(trans),
                    ..Default::default()
                },
                DigitSprite,
                MwTilePos(coord.0),
            )).id();
            commands.entity(e).insert(TileDigitEntity(e_digit));
        }
    }
}

fn tile_gent_sprite_mgr(
    mut commands: Commands,
    settings: Res<AllSettings>,
    assets: Res<GameAssets>,
    viewing: Res<PlidViewing>,
    q_tile: Query<
        (Entity, &MwTilePos, &TileGent, &Transform, Option<&TileGentEntity>),
        (With<BaseSprite>, Changed<TileGent>)
    >,
    mut q_gent: Query<&mut TextureAtlasSprite, With<GentSprite>>,
) {
    for (e, coord, gent, xf, spr_gent) in q_tile.iter() {
        let mut trans = xf.translation;
        trans.z = zpos::GENTS;

        let (i_gent, clr_gent) = match gent {
            TileGent::Empty |
            TileGent::Item(ItemKind::Safe) |
            TileGent::Flag(PlayerId::Neutral)=> {
                if let Some(spr_gent) = spr_gent {
                    commands.entity(spr_gent.0).despawn();
                    commands.entity(e).remove::<TileGentEntity>();
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

        if let Some(spr_gent) = spr_gent {
            // there is an existing gent entity we can reuse
            let e_gent = spr_gent.0;
            let mut sprite = q_gent.get_mut(e_gent).unwrap();
            sprite.index = i_gent;
            sprite.color = clr_gent;
        } else {
            // create a new gent entity
            let e_gent = commands.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: i_gent,
                        color: clr_gent,
                        ..Default::default()
                    },
                    texture_atlas: assets.sprites.clone(),
                    transform: Transform::from_translation(trans),
                    ..Default::default()
                },
                GentSprite,
                MwTilePos(coord.0),
            )).id();
            commands.entity(e).insert(TileGentEntity(e_gent));
        }
    }
}

fn explosion_sprite_mgr(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<GameAssets>,
    mut q_expl: Query<(
        Entity, &TileExplosion, Option<&mut ExplosionSprite>, Option<&mut TextureAtlasSprite>,
    )>,
    q_tile: Query<&Transform, With<BaseSprite>>,
) {
    for (e, expl, spr_expl, sprite) in &mut q_expl {
        if let (Some(mut spr_expl), Some(mut sprite)) = (spr_expl, sprite) {
            // sprite already set up, manage it
            spr_expl.timer.tick(time.delta());
            if spr_expl.timer.finished() {
                commands.entity(e).despawn_recursive();
            }
            sprite.color.set_a(spr_expl.timer.percent_left());
        } else {
            // we have an entity with no sprite, set up the sprite
            let xf = q_tile.get(expl.0).unwrap();
            let mut trans = xf.translation;
            trans.z = zpos::OVERLAYS;
            let i_expl = match expl.1 {
                TileExplosionKind::Normal => super::sprite::EXPLOSION_MINE,
                TileExplosionKind::Decoy => super::sprite::EXPLOSION_DECOY,
            };
            commands.entity(e).insert((
                ExplosionSprite {
                    timer: Timer::new(Duration::from_millis(1000), TimerMode::Once),
                },
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: i_expl,
                        ..Default::default()
                    },
                    texture_atlas: assets.sprites.clone(),
                    transform: Transform::from_translation(trans),
                    ..Default::default()
                },
            ));
        }
    }
}

fn sprites_reghighlight(
    mut commands: Commands,
    settings: Res<AllSettings>,
    assets: Res<GameAssets>,
    mapdesc: Res<MapDescriptor>,
    cits: Res<CitIndex>,
    cursor_tile: Res<GridCursorTileEntity>,
    q_highlight: Query<Entity, With<RegHighlightSprite>>,
    q_tile: Query<(&Transform, &TileRegion), With<BaseSprite>>,
    q_cit: Query<&CitOwner>,
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
            for e in &q_highlight {
                commands.entity(e).despawn_recursive();
            }
            // create new
            let index = match mapdesc.topology {
                Topology::Hex => super::sprite::TILES6 + super::sprite::TILE_HIGHLIGHT,
                Topology::Sq => super::sprite::TILES4 + super::sprite::TILE_HIGHLIGHT,
            };
            let color = if let Some(e_cit) = cits.by_id.get(region as usize) {
                let owner = q_cit.get(*e_cit).unwrap().0;
                let mut lcha = settings.player_colors.visible[owner.i()];
                lcha.0 *= 0.75;
                lcha.1 *= 0.75;
                Color::from(lcha)
            } else {
                return;
            };
            for (xf, tile_region) in &q_tile {
                let mut trans = xf.translation;
                trans.z = zpos::REGHILIGHT;
                if tile_region.0 == region {
                    commands.spawn((
                        RegHighlightSprite,
                        SpriteSheetBundle {
                            sprite: TextureAtlasSprite {
                                index,
                                color,
                                ..Default::default()
                            },
                            texture_atlas: assets.sprites.clone(),
                            transform: Transform::from_translation(trans),
                            ..Default::default()
                        },
                    ));
                }
            }
        }
    } else {
        for e in &q_highlight {
            commands.entity(e).despawn_recursive();
        }
    }
}
