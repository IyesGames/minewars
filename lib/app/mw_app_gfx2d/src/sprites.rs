use mw_app_core::{assets::SpritesAssets, camera::ActiveGameCamera, map::{tile::*, *}, player::{Plid, PlidColor}, session::{NeedsSessionGovernorSet, PlayersIndex, SessionGovernor}, settings::PlidColorSettings};
use mw_common::grid::*;

use crate::{misc::*, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        setup_tile_entities
            .track_progress()
            .in_set(SetStage::Provide(TileUpdateSS))
            .in_set(Gfx2dImplSet::Sprites)
            .in_set(NeedsMapGovernorSet),
    )
        .in_set(InStateSet(AppState::GameLoading)),
    );
    app.add_systems(OnEnter(AppState::InGame), (
        reveal_sprites_onenter_ingame
            .in_set(Gfx2dImplSet::Sprites),
        setup_cursor
            .in_set(Gfx2dImplSet::Any),
    ));
    app.add_systems(Update, (
        update_billboard_sprites,
    )
        .in_set(Gfx2dImplSet::Sprites)
    );
    app.add_systems(Update, (
        update_sprite_tile_kind,
        update_sprite_tile_owner
            .in_set(NeedsSessionGovernorSet),
    )
        .in_set(SetStage::WantChanged(TileUpdateSS))
        .in_set(Gfx2dImplSet::Sprites)
        .in_set(NeedsMapGovernorSet)
    );
    app.add_systems(Update, (
        cursor_update,
    )
        .in_set(SetStage::WantChanged(GridCursorSS))
        .in_set(Gfx2dImplSet::Any)
        .in_set(NeedsMapGovernorSet)
    );
}

#[derive(Component)]
struct SpriteEntities {
    base: Entity,
    base_overlay: Option<Entity>,
    digit: Option<Entity>,
}

#[derive(Component)]
struct TileEntity(Entity);

#[derive(Component)]
struct MapSprite;

#[derive(Component)]
struct BillboardSprite;

#[derive(Component)]
pub struct CursorSprite;
#[derive(Component)]
pub struct BaseSprite;
#[derive(Component)]
pub struct BaseOverlaySprite;
#[derive(Component)]
pub struct DigitSprite;
#[derive(Component)]
pub struct GentSprite;
#[derive(Component)]
pub struct RegHighlightSprite;
#[derive(Component)]
pub struct ExplosionSprite {
    pub timer: Timer,
}

#[derive(Bundle)]
struct MapSpriteBundle {
    cleanup: GamePartialCleanup,
    marker: MapSprite,
    pos: MwTilePos,
    tile: TileEntity,
    sprite: SpriteBundle,
    atlas: TextureAtlas,
}

#[derive(Bundle)]
struct BaseSpriteBundle {
    mapsprite: MapSpriteBundle,
    marker: BaseSprite,
}

#[derive(Bundle)]
struct BaseOverlaySpriteBundle {
    mapsprite: MapSpriteBundle,
    marker: BaseOverlaySprite,
}

#[derive(Bundle)]
struct CursorSpriteBundle {
    cleanup: GamePartialCleanup,
    marker: CursorSprite,
    sprite: SpriteBundle,
    atlas: TextureAtlas,
    pos: MwTilePos,
}

fn setup_cursor(
    mut commands: Commands,
    assets: Res<SpritesAssets>,
    q_map: Query<&MapDescriptor, With<MapGovernor>>,
) {
    let mapdesc = q_map.single();
    let i = match mapdesc.topology {
        Topology::Hex => sprite::TILES6 + sprite::TILE_CURSOR,
        Topology::Sq => sprite::TILES4 + sprite::TILE_CURSOR,
    };
    commands.spawn((
        CursorSpriteBundle {
            cleanup: GamePartialCleanup,
            sprite: SpriteBundle {
                texture: assets.tiles_img.clone(),
                transform: Transform::from_xyz(0.0, 0.0, zpos::CURSOR),
                ..Default::default()
            },
            atlas: TextureAtlas {
                index: i,
                layout: assets.tiles_layout.clone(),
            },
            pos: MwTilePos(Pos::origin()),
            marker: CursorSprite,
        },
    ));
}

fn cursor_update(
    q_map: Query<(&MapDescriptor, &GridCursor), With<MapGovernor>>,
    mut q: Query<(&mut Transform, &mut MwTilePos), With<CursorSprite>>,
) {
    let (desc, crs) = q_map.single();

    let Some(c) = crs.0 else {
        return;
    };
    let Ok((mut xf, mut pos)) = q.get_single_mut() else {
        return;
    };

    *pos = MwTilePos(c);

    let (width, height, trans) = match desc.topology {
        Topology::Hex => (
            sprite::WIDTH6, sprite::HEIGHT6,
            Hex::from(c).translation(),
        ),
        Topology::Sq => (
            sprite::WIDTH4, sprite::HEIGHT4,
            Sq::from(c).translation(),
        ),
    };
    xf.translation = Vec3::new(trans.x * width, trans.y * height, zpos::CURSOR);
}

fn setup_tile_entities(
    mut commands: Commands,
    spreader: Res<WorkSpreader>,
    assets: Res<SpritesAssets>,
    mut q_map: Query<(Entity, &mut TileUpdateQueue, &MapDescriptor, &MapTileIndex), With<MapGovernor>>,
    mut done: Local<bool>,
) -> Progress {
    if *done {
        return true.into();
    }
    let Ok((e_map, mut tuq, desc, tile_index)) = q_map.get_single_mut() else {
        return false.into();
    };
    if spreader.acquire() {
        return false.into();
    }
    tuq.queue_all();

    let (base_i, width, height) = match desc.topology {
        Topology::Hex => (
            sprite::TILES6 + sprite::TILE_LAND,
            sprite::WIDTH6, sprite::HEIGHT6,
        ),
        Topology::Sq => (
            sprite::TILES4 + sprite::TILE_LAND,
            sprite::WIDTH4, sprite::HEIGHT4,
        ),
    };

    for (c, &e_tile) in tile_index.0.iter() {
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
        let e_spr = commands.spawn(BaseSpriteBundle {
            marker: BaseSprite,
            mapsprite: MapSpriteBundle {
                cleanup: GamePartialCleanup,
                marker: MapSprite,
                tile: TileEntity(e_tile),
                sprite: SpriteBundle {
                    texture: assets.tiles_img.clone(),
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
                atlas: TextureAtlas {
                    index: base_i,
                    layout: assets.tiles_layout.clone(),
                },
                pos: MwTilePos(c.into()),
            },
        }).id();
        commands.entity(e_tile).insert(SpriteEntities {
            base: e_spr,
            base_overlay: None,
            digit: None,
        });
    }

    *done = true;

    debug!("Initialized map graphics using 2D Sprites.");

    false.into()
}

fn reveal_sprites_onenter_ingame(
    mut q_sprite: Query<&mut Visibility, With<MapSprite>>,
) {
    for mut vis in &mut q_sprite {
        *vis = Visibility::Visible;
    }
}

fn update_billboard_sprites(
    q_camera: Query<&Transform, (With<Camera2d>, With<ActiveGameCamera>, Changed<Transform>, Without<BillboardSprite>)>,
    mut q_sprite: Query<&mut Transform, With<BillboardSprite>>,
) {
    let Ok(xf_cam) = q_camera.get_single() else {
        return;
    };
    q_sprite.iter_mut().for_each(|mut xf_spr| {
        xf_spr.rotation = -xf_cam.rotation;
    })
}

fn update_sprite_tile_kind(
    mut commands: Commands,
    assets: Res<SpritesAssets>,
    q_map: Query<(&TileUpdateQueue, &MapDescriptor), With<MapGovernor>>,
    mut q_tile: Query<(Entity, &MwTilePos, &TileKind, &mut SpriteEntities), With<MwMapTile>>,
    mut q_sprite: Query<(&mut TextureAtlas, Has<BillboardSprite>), With<BaseSprite>>,
    mut q_overlay: Query<(&mut TextureAtlas, Has<BillboardSprite>), (With<BaseOverlaySprite>, Without<BaseSprite>)>,
) {
    let (tuq, desc) = q_map.single();
    let i_kind = match desc.topology {
        Topology::Hex => sprite::TILES6,
        Topology::Sq => sprite::TILES4,
    };
    tuq.for_each(&mut q_tile, |(e_tile, pos, kind, mut e_spr)| {
        if let Ok((mut atlas, is_billboard)) = q_sprite.get_mut(e_spr.base) {
            let (i_base, base_billboard, i_overlay, overlay_billboard) = match kind {
                TileKind::Water =>
                    (sprite::TILE_WATER, true, None, false),
                TileKind::FoundationRoad =>
                    (sprite::TILE_FOUNDATION, false, None, false),
                TileKind::FoundationStruct =>
                    (sprite::TILE_FOUNDATION, false, None, false),
                TileKind::Regular =>
                    (sprite::TILE_LAND, false, None, false),
                TileKind::Fertile =>
                    (sprite::TILE_LAND, false, Some(sprite::TILE_FERTILE), true),
                TileKind::Forest =>
                    (sprite::TILE_LAND, false, Some(sprite::TILE_FOREST), true),
                TileKind::Mountain =>
                    (sprite::TILE_LAND, false, Some(sprite::TILE_MTN), true),
                TileKind::Destroyed =>
                    (sprite::TILE_DEAD, false, None, false),
            };
            atlas.index = i_kind + i_base;
            if is_billboard && !base_billboard {
                commands.entity(e_spr.base).remove::<BillboardSprite>();
            }
            if !is_billboard && base_billboard {
                commands.entity(e_spr.base).insert(BillboardSprite);
            }
            match (e_spr.base_overlay, i_overlay) {
                (None, None) => {},
                (Some(e_overlay), None) => {
                    commands.entity(e_overlay).despawn_recursive();
                }
                (None, Some(i_overlay)) => {
                    let e = commands.spawn(BaseOverlaySpriteBundle {
                        marker: BaseOverlaySprite,
                        mapsprite: MapSpriteBundle {
                            cleanup: GamePartialCleanup,
                            marker: MapSprite,
                            pos: pos.clone(),
                            tile: TileEntity(e_tile),
                            sprite: SpriteBundle {
                                texture: assets.tiles_img.clone(),
                                transform: Transform::from_xyz(0.0, 0.0, 0.5),
                                ..Default::default()
                            },
                            atlas: TextureAtlas {
                                layout: assets.tiles_layout.clone(),
                                index: i_kind + i_overlay,
                            },
                        },
                    }).id();
                    if overlay_billboard {
                        commands.entity(e).insert(BillboardSprite);
                    }
                    commands.entity(e_spr.base).add_child(e);
                    e_spr.base_overlay = Some(e);
                }
                (Some(e_overlay), Some(i_overlay)) => {
                    let (mut atlas, is_billboard) = q_overlay.get_mut(e_overlay).unwrap();
                    atlas.index = i_kind + i_overlay;
                    if is_billboard && !overlay_billboard {
                        commands.entity(e_overlay).remove::<BillboardSprite>();
                    }
                    if !is_billboard && overlay_billboard {
                        commands.entity(e_overlay).insert(BillboardSprite);
                    }
                }
            }
        }
    });
}

fn update_sprite_tile_owner(
    settings: Settings,
    q_map: Query<(&TileUpdateQueue,), With<MapGovernor>>,
    mut q_tile: Query<(&MwTilePos, &TileOwner, &SpriteEntities), With<MwMapTile>>,
    mut q_sprite: Query<&mut Sprite, With<BaseSprite>>,
    q_player: Query<&PlidColor, With<Plid>>,
    q_session: Query<&PlayersIndex, With<SessionGovernor>>,
) {
    let s_colors = settings.get::<PlidColorSettings>().unwrap();
    let color_neutral = s_colors.colors[0];
    let (tuq,) = q_map.single();
    let players_index = q_session.single();
    tuq.for_each(&mut q_tile, |(pos, owner, e_spr)| {
        let color = players_index.e_plid.get(owner.0.i())
            .and_then(|e| q_player.get(*e).ok())
            .map(|plidcolor| plidcolor.color)
            .unwrap_or(color_neutral.into());
        if let Ok(mut sprite) = q_sprite.get_mut(e_spr.base) {
            sprite.color = color;
        }
    });
}
