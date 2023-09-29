use crate::assets::GameAssets;
use crate::camera::GridCursor;
use crate::camera::GridCursorSet;
use crate::prelude::*;

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
                setup_base_tile::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                setup_base_tile::<Sq>
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
                tile_flag_sprite_mgr.after(MapUpdateSet::TileFlag),
                tile_gent_sprite_mgr.after(MapUpdateSet::TileGent),
                explosion_sprite_mgr,
            )
                .run_if(resource_exists::<TilemapInitted>()),
        ).in_set(Gfx2dSet::Sprites));
        app.add_systems(OnEnter(AppState::InGame), (
            setup_cursor,
        ));
        app.add_systems(Update, (
            (
                cursor_sprite::<Hex>
                    .in_set(MapTopologySet(Topology::Hex)),
                cursor_sprite::<Sq>
                    .in_set(MapTopologySet(Topology::Sq)),
            )
                .after(GridCursorSet),
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
#[derive(Component)]
#[component(storage = "SparseSet")]
struct TileFlagEntity(Entity);

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
    if !crs.is_changed() {
        return;
    }
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
        trans.z += zpos::DIGIT;

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

fn tile_flag_sprite_mgr(
    settings: Res<AllSettings>,
    viewing: Res<PlidViewing>,
    mut commands: Commands,
    assets: Res<GameAssets>,
    q_tile: Query<
        (Entity, &MwTilePos, &TileFlag, &Transform, Option<&TileFlagEntity>),
        (With<BaseSprite>, Changed<TileFlag>)
    >,
    mut q_flag: Query<&mut TextureAtlasSprite, With<FlagSprite>>,
) {
    for (e, coord, flag, xf, spr_flag) in q_tile.iter() {
        let mut trans = xf.translation;
        trans.z += zpos::OVERLAYS;

        let (i_flag, color) = if flag.0 == viewing.0 {
            (
                super::sprite::FLAGS + settings.player_colors.flag_style as usize,
                Color::WHITE,
            )
        } else {
            (
                super::sprite::FLAGS,
                settings.player_colors.visible[flag.0.i()].into(),
            )
        };

        if let Some(spr_flag) = spr_flag {
            // there is an existing flag entity we can reuse (or despawn)
            if flag.0 != PlayerId::Neutral {
                let e_flag = spr_flag.0;
                let mut sprite = q_flag.get_mut(e_flag).unwrap();
                sprite.index = i_flag;
                sprite.color = color;
            } else {
                commands.entity(spr_flag.0).despawn();
                commands.entity(e).remove::<TileFlagEntity>();
            }
        } else if flag.0 != PlayerId::Neutral {
            // create a new flag entity
            let e_flag = commands.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: i_flag,
                        color,
                        ..Default::default()
                    },
                    texture_atlas: assets.sprites.clone(),
                    transform: Transform::from_translation(trans),
                    ..Default::default()
                },
                FlagSprite,
                MwTilePos(coord.0),
            )).id();
            commands.entity(e).insert(TileFlagEntity(e_flag));
        }
    }
}

fn tile_gent_sprite_mgr(
    mut commands: Commands,
    assets: Res<GameAssets>,
    q_tile: Query<
        (Entity, &MwTilePos, &TileGent, &Transform, Option<&TileGentEntity>),
        (With<BaseSprite>, Changed<TileGent>)
    >,
    mut q_gent: Query<&mut TextureAtlasSprite, With<GentSprite>>,
) {
    for (e, coord, gent, xf, spr_gent) in q_tile.iter() {
        let mut trans = xf.translation;
        trans.z += zpos::DIGIT;

        let i_gent = match gent {
            TileGent::Empty | TileGent::Item(ItemKind::Safe) => {
                if let Some(spr_gent) = spr_gent {
                    commands.entity(spr_gent.0).despawn();
                    commands.entity(e).remove::<TileGentEntity>();
                }
                continue;
            }
            TileGent::Item(kind) => {
                match kind {
                    ItemKind::Mine => super::sprite::GENT_MINE,
                    ItemKind::Decoy => super::sprite::GENT_DECOY,
                    ItemKind::Flashbang => super::sprite::GENT_FLASH,
                    ItemKind::Safe => unreachable!(),
                }
            }
            TileGent::Structure(kind) => {
                match kind {
                    StructureKind::Barricade => super::sprite::GENT_WALL,
                    StructureKind::WatchTower => super::sprite::GENT_TOWER,
                    StructureKind::Bridge => todo!(),
                    StructureKind::Road => panic!("Roads must use TileRoads, not TileGent"),
                }
            }
            TileGent::Cit(_) => super::sprite::GENT_CIT,
        };

        if let Some(spr_gent) = spr_gent {
            // there is an existing digit entity we can reuse
            let e_gent = spr_gent.0;
            let mut sprite = q_gent.get_mut(e_gent).unwrap();
            sprite.index = i_gent;
        } else {
            // create a new gent entity
            let e_gent = commands.spawn((
                SpriteSheetBundle {
                    sprite: TextureAtlasSprite {
                        index: i_gent,
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
            trans.z += zpos::OVERLAYS;
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

