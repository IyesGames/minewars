use crate::assets::{TileAssets, ZoomLevelDescriptor};
use crate::map::MaxViewBounds;
use crate::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseScrollUnit, MouseMotion};
use bevy::render::camera::RenderTarget;
use bevy_tweening::*;
use bevy_tweening::lens::*;
use mw_common::game::MapDescriptor;
use mw_common::grid::{Pos, Topology, Hex, Sq, Sqr, Coord};

use crate::AppGlobalState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldCursor(Vec2::ZERO));
        app.insert_resource(WorldCursorPrev(Vec2::ZERO));
        app.insert_resource(GridCursor(Pos(0, 0)));
        app.add_system_to_stage(CoreStage::PreUpdate, world_cursor_system);
        app.add_enter_system(AppGlobalState::InGame, setup_camera);
        app.add_enter_system(AppGlobalState::GameLoading, setup_zoom);
        app.add_exit_system(AppGlobalState::InGame, despawn_with_recursive::<CameraCleanup>);
        app.add_system(
            camera_control_zoom_mousewheel
                .run_in_state(AppGlobalState::InGame)
                .before("zoom")
        );
        app.add_system(
            camera_control_pan_mousedrag
                .run_in_state(AppGlobalState::InGame)
                .after("zoom")
        );
        app.add_system(
            apply_next_zoomlevel
                .run_in_state(AppGlobalState::InGame)
                .label("zoom")
                .after(bevy_tweening::AnimationSystem::AnimationUpdate)
        );
        app.add_system(
            grid_cursor
                .run_in_state(AppGlobalState::InGame)
                .label("cursor")
        );
    }
}

#[derive(Component)]
struct CameraCleanup;

#[derive(Component)]
struct GameCamera;

fn setup_zoom(
    mut commands: Commands,
    tiles: Res<TileAssets>,
) {
    commands.insert_resource(ZoomLevel {
        i: 0,
        desc: tiles.zoomlevels.zoom[0].clone(),
    });
}

fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn_bundle(Camera2dBundle::default())
        .insert(CameraZoomLevel::default())
        .insert(CameraCleanup)
        .insert(GameCamera);
}

/// The current camera zoom level
///
/// (see `TileAssets`)
pub struct ZoomLevel {
    pub i: usize,
    pub desc: ZoomLevelDescriptor,
}

/// State for keeping track of transitions between zoomlevels
#[derive(Component, Default)]
pub struct CameraZoomLevel {
    current: usize,
    next: usize,
}

fn camera_control_zoom_mousewheel(
    mut wheel: EventReader<MouseWheel>,
    tiles: Res<TileAssets>,
    mut q: Query<&mut CameraZoomLevel, With<GameCamera>>,
) {
    let mut change = 0.0;

    // accumulate all events into one variable
    for ev in wheel.iter() {
        let delta = match ev.unit {
            MouseScrollUnit::Line => -ev.y,
            MouseScrollUnit::Pixel => unimplemented!(),
        };
        change += delta;
    }

    if change != 0.0 {
        let mut level = q.single_mut();

        let change = change as isize;
        let mut l = level.next as isize;
        l += change;
        l = l.clamp(0, tiles.zoomlevels.zoom.len() as isize - 1);
        level.next = l as usize;
    }
}

fn apply_next_zoomlevel(
    mut commands: Commands,
    tiles: Res<TileAssets>,
    wcrs: Res<WorldCursor>,
    descriptor: Res<MapDescriptor>,
    mut res_zoom: ResMut<ZoomLevel>,
    mut q_cam: Query<(Entity, &mut Transform, &mut CameraZoomLevel), (With<GameCamera>, Changed<CameraZoomLevel>)>,
    mut q_spr: Query<(&mut Sprite, &mut Transform, &mut Handle<Image>, &TilePos), (Without<GameCamera>, Without<TilemapTexture>)>,
    mut q_tmap: Query<(&mut TilemapTexture, &mut Transform, &mut TilemapGridSize, &mut TilemapTileSize), (Without<GameCamera>, Without<Sprite>)>,
) {
    const JUMP_DUR: Duration = Duration::from_millis(125);

    let (e_cam, mut xf_cam, mut zoom) = if let Ok(x) = q_cam.get_single_mut() {
        x
    } else {
        return;
    };

    let zoom_old = &tiles.zoomlevels.zoom[zoom.current];
    let zoom_new = &tiles.zoomlevels.zoom[zoom.next];

    for (mut spr, mut xf, mut handle, pos) in q_spr.iter_mut() {
        if *handle == tiles.gents[zoom.current] {
            *handle = tiles.gents[zoom.next].clone();
        } else if *handle == tiles.tiles6[zoom.current] {
            *handle = tiles.tiles6[zoom.next].clone();
        } else if *handle == tiles.tiles4[zoom.current] {
            *handle = tiles.tiles4[zoom.next].clone();
        } else if *handle == tiles.roads6[zoom.current] {
            *handle = tiles.roads6[zoom.next].clone();
        } else if *handle == tiles.roads4[zoom.current] {
            *handle = tiles.roads4[zoom.next].clone();
        } else if *handle == tiles.digits[zoom.current] {
            *handle = tiles.digits[zoom.next].clone();
        } else if *handle == tiles.flags[zoom.current] {
            *handle = tiles.flags[zoom.next].clone();
        } else {
            continue;
        }

        if let Some(rect) = &mut spr.rect {
            rect.min /= zoom_old.size as f32;
            rect.min *= zoom_new.size as f32;
            rect.max /= zoom_old.size as f32;
            rect.max *= zoom_new.size as f32;
        }
        let trans = translation_pos(descriptor.topology, pos.into(), &zoom_new);
        xf.translation = trans.extend(xf.translation.z);
    }

    for (mut tm_tex, mut tm_xf, mut tm_gsz, mut tm_tsz) in q_tmap.iter_mut() {
        if tm_tex.0 == tiles.gents[zoom.current] {
            tm_tex.0 = tiles.gents[zoom.next].clone();
        } else if tm_tex.0 == tiles.tiles6[zoom.current] {
            tm_tex.0 = tiles.tiles6[zoom.next].clone();
        } else if tm_tex.0 == tiles.tiles4[zoom.current] {
            tm_tex.0 = tiles.tiles4[zoom.next].clone();
        } else if tm_tex.0 == tiles.roads6[zoom.current] {
            tm_tex.0 = tiles.roads6[zoom.next].clone();
        } else if tm_tex.0 == tiles.roads4[zoom.current] {
            tm_tex.0 = tiles.roads4[zoom.next].clone();
        } else if tm_tex.0 == tiles.digits[zoom.current] {
            tm_tex.0 = tiles.digits[zoom.next].clone();
        } else if tm_tex.0 == tiles.flags[zoom.current] {
            tm_tex.0 = tiles.flags[zoom.next].clone();
        } else {
            continue;
        }
        *tm_gsz = match descriptor.topology {
            Topology::Hex => TilemapGridSize { x: zoom_new.offset6.0 as f32, y: zoom_new.offset6.1 as f32 },
            Topology::Sq | Topology::Sqr => TilemapGridSize { x: zoom_new.offset4.0 as f32, y: zoom_new.offset4.1 as f32 },
        };
        *tm_tsz = TilemapTileSize { x: zoom_new.size as f32, y: zoom_new.size as f32 };
        let trans = translation_tmap(descriptor.topology, &zoom_new);
        tm_xf.translation = trans.extend(tm_xf.translation.z);
    }

    let (offset_old, offset_new): (Vec2, Vec2) = match descriptor.topology {
        Topology::Hex => (zoom_old.offset6.into(), zoom_new.offset6.into()),
        Topology::Sq | Topology::Sqr => (zoom_old.offset4.into(), zoom_new.offset4.into()),
    };

    // we need to set the camera scale as to start the animation
    // from something that looks like the old zoom level
    // let scale = zoom_new.size as f32 / zoom_old.size as f32;
    xf_cam.scale.x = xf_cam.scale.x / zoom_old.size as f32 * zoom_new.size as f32;
    xf_cam.scale.y = xf_cam.scale.y / zoom_old.size as f32 * zoom_new.size as f32;
    // xf_cam.scale = Vec3::new(scale, scale, xf_cam.scale.z);

    // different offsets at each zoom level mean that locations on the map might not correspond
    // we need to jump to the "current camera location" but in the new zoom level
    // and then animate to the "current cursor location" in the new zoom level
    let cam_xy = xf_cam.translation.truncate();
    let fix_xy = (cam_xy / offset_old * offset_new).round();
    xf_cam.translation = fix_xy.extend(xf_cam.translation.z);
    // let tgt_xy = wcrs.0 / offset_old * offset_new;

    let anim = Animator::new(Tween::new(
        EaseFunction::QuadraticOut,
        TweeningType::Once,
        JUMP_DUR,
        TransformPositionScaleLens {
            pos_start: xf_cam.translation,
            pos_end: fix_xy.extend(xf_cam.translation.z),
            scale_start: xf_cam.scale,
            scale_end: Vec3::new(1.0, 1.0, xf_cam.scale.z),
        }
    ));
    commands.entity(e_cam).insert(anim);

    xf_cam.translation.round();
    zoom.current = zoom.next;
    *res_zoom = ZoomLevel {
        i: zoom.next,
        desc: zoom_new.clone(),
    }
}

/// Because bevy_tweening is stupid and only supports either position or scale
struct TransformPositionScaleLens {
    pos_start: Vec3,
    pos_end: Vec3,
    scale_start: Vec3,
    scale_end: Vec3,
}
impl Lens<Transform> for TransformPositionScaleLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let pos_value = self.pos_start + (self.pos_end - self.pos_start) * ratio;
        target.translation = pos_value;
        let scale_value = self.scale_start + (self.scale_end - self.scale_start) * ratio;
        target.scale = scale_value;
    }
}

pub struct WorldCursor(pub Vec2);
pub struct WorldCursorPrev(pub Vec2);
pub struct GridCursor(pub Pos);

fn world_cursor_system(
    mut crs: ResMut<WorldCursor>,
    mut crs_old: ResMut<WorldCursorPrev>,
    // need to get window dimensions
    wnds: Res<Windows>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>
) {
    // get the camera info and transform
    // assuming there is exactly one main camera entity, so query::single() is OK
    if let Ok((camera, camera_transform)) = q_camera.get_single() {
        // get the window that the camera is displaying to (or the primary window)
        let wnd = if let RenderTarget::Window(id) = camera.target {
            wnds.get(id).unwrap()
        } else {
            wnds.get_primary().unwrap()
        };

        // check if the cursor is inside the window and get its position
        if let Some(screen_pos) = wnd.cursor_position() {
            // get the size of the window
            let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);

            // convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
            let ndc = (screen_pos / window_size) * 2.0 - Vec2::ONE;

            // matrix for undoing the projection and camera transform
            let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix().inverse();

            // use it to convert ndc to world-space coordinates
            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

            // reduce it to a 2D value
            let world_pos: Vec2 = world_pos.truncate();

            crs_old.0 = crs.0;
            crs.0 = world_pos;
        }
    }
}

fn grid_cursor(
    crs_in: Res<WorldCursor>,
    mut crs_out: ResMut<GridCursor>,
    mapdesc: Res<MapDescriptor>,
    zoom: Res<ZoomLevel>,
) {
    crs_out.0 = match mapdesc.topology {
        Topology::Hex => {
            let tdim = Vec2::new(zoom.desc.offset6.0 as f32, zoom.desc.offset6.1 as f32);
            // PERF: fugly
            let conv = bevy_math::Mat2::from_cols_array(
                &[tdim.x, 0.0, tdim.x * 0.5, tdim.y * 0.75]
            ).inverse();
            let grid = conv * crs_in.0;
            Hex::from_f32_clamped(grid.into()).into()
        }
        Topology::Sq => {
            let tdim = Vec2::new(zoom.desc.offset4.0 as f32, zoom.desc.offset4.1 as f32);
            let adj = crs_in.0 / tdim;
            Sq::from_f32_clamped(adj.into()).into()
        }
        Topology::Sqr => {
            let tdim = Vec2::new(zoom.desc.offset4.0 as f32, zoom.desc.offset4.1 as f32);
            let adj = crs_in.0 / tdim;
            Sqr::from_f32_clamped(adj.into()).into()
        }
    };
}

pub fn translation_pos(topology: Topology, pos: Pos, zoom: &ZoomLevelDescriptor) -> Vec2 {
    match topology {
        Topology::Hex => {
            Hex::from(pos).translation() * Vec2::new(zoom.offset6.0 as f32, zoom.offset6.1 as f32)
        }
        Topology::Sq | Topology::Sqr => {
            Sq::from(pos).translation() * Vec2::new(zoom.offset4.0 as f32, zoom.offset4.1 as f32)
        }
    }.floor()
}

pub fn translation_tmap(topology: Topology, zoom: &ZoomLevelDescriptor) -> Vec2 {
    match topology {
        Topology::Hex => {
            Vec2::new(
                - 128.0 * zoom.offset6.0 as f32 * 1.5 - zoom.size as f32 * 0.5,
                - 128.0 * zoom.offset6.1 as f32 * 0.75 - zoom.size as f32 * 0.5,
            )
        }
        Topology::Sq | Topology::Sqr => {
            Vec2::new(
                - 128.0 * zoom.offset4.0 as f32 - zoom.size as f32 * 0.5,
                - 128.0 * zoom.offset4.1 as f32 - zoom.size as f32 * 0.5,
            )
        }
    }.floor()
}

fn camera_control_pan_mousedrag(
    btn: Res<Input<MouseButton>>,
    mut motion: EventReader<MouseMotion>,
    mut q_camera: Query<&mut Transform, With<GameCamera>>,
    bounds: Option<Res<MaxViewBounds>>,
) {
    if btn.pressed(MouseButton::Right) {
        let mut delta = Vec2::ZERO;

        for ev in motion.iter() {
            delta += ev.delta;
        }

        if delta != Vec2::ZERO {
            let mut cam = q_camera.single_mut();
            cam.translation.x -= delta.x * cam.scale.x;
            cam.translation.y += delta.y * cam.scale.y;

/*
            if let Some(bounds) = bounds {
                let mut cam_xy = cam.translation.truncate();
                let r = cam_xy.length();
                if r > bounds.0 {
                    cam_xy = cam_xy.normalize() * bounds.0;
                    cam.translation.x = cam_xy.x;
                    cam.translation.y = cam_xy.y;
                }
            }
*/
        }
    }
    if btn.just_released(MouseButton::Right) {
        let mut xf_cam = q_camera.single_mut();
        // round camera translation to a full pixel at our current zoom level
        // (so rendering looks nice)
        let xy = xf_cam.translation.truncate();
        xf_cam.translation = xy.round().extend(xf_cam.translation.z);
    }
}
