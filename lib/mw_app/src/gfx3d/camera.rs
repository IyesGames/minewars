use bevy::{input::mouse::MouseScrollUnit, render::camera::ScalingMode, window::PrimaryWindow};
use mw_common::{game::MapDescriptor, grid::*};

use crate::map::{GridCursorTileEntity, MapTileIndex, MapTopologySet};
use crate::{prelude::*, ui::UiCamera};
use crate::camera::{CameraControlSS, GameCamera, GridCursor, GridCursorSS};

use super::*;

pub struct Gfx3dCameraPlugin;

impl Plugin for Gfx3dCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_game_camera_3d.in_set(Gfx3dModeSet::Any));
        app.add_systems(Update, (
            cursor_to_ground_plane
                .in_set(InStateSet(AppState::InGame))
                .in_set(Gfx3dModeSet::Any)
                .in_set(SetStage::Provide(WorldCursorSS)),
            (
                grid_cursor::<Hex>.in_set(MapTopologySet(Topology::Hex)),
                grid_cursor::<Sq>.in_set(MapTopologySet(Topology::Sq)),
            )
                .in_set(Gfx3dModeSet::Any)
                .in_set(SetStage::Provide(GridCursorSS))
                .in_set(SetStage::WantChanged(WorldCursorSS)),
            pan_orbit_camera
                .in_set(Gfx3dModeSet::Any)
                .in_set(SetStage::Provide(CameraControlSS))
                .in_set(SetStage::Prepare(WorldCursorSS)),
        ));
    }
}

#[derive(Component)]
struct PanOrbitCamera {
    center: Vec2,
    radius: f32,
    yaw: f32,
    pitch: f32,
}

fn setup_game_camera_3d(
    mut commands: Commands,
) {
    // let projection = OrthographicProjection {
    //     near: 0.0,
    //     far: RENDER_RANGE,
    //     scaling_mode: ScalingMode::FixedVertical(TILE_SCALE),
    //     scale: 24.0,
    //     ..Default::default()
    // };
    // let mut transform = Transform::from_xyz(0.0, 0.0, -RENDER_RANGE / 2.0);
    // transform.rotate_x(-30f32.to_radians());
    // transform.rotate_y( 45f32.to_radians());
    let transform = Transform::from_xyz(600.0, 800.0, -800.0)
        .looking_at(Vec3::ZERO, Vec3::Y);
    commands.spawn((
        StateDespawnMarker,
        GameCamera,
        UiCamera,
        Camera3dBundle {
            // projection: projection.into(),
            transform,
            ..Default::default()
        },
        PanOrbitCamera {
            center: Vec2::ZERO,
            radius: 1600.0,
            yaw: 0.0,
            pitch: 0.75,
        }
    ));
    commands.spawn((
        DirectionalLightBundle {
            directional_light: DirectionalLight {
                illuminance: 10_000.0,
                ..Default::default()
            },
            transform: Transform::from_xyz(-100.0, 200.0, 100.0)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
    ));
}

fn pan_orbit_camera(
    kbd: Res<ButtonInput<KeyCode>>,
    mousebtn: Res<ButtonInput<MouseButton>>,
    mut ev_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mut ev_scroll: EventReader<bevy::input::mouse::MouseWheel>,
    mut q_camera: Query<(&mut PanOrbitCamera, &mut Transform, &Camera)>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut init: Local<bool>,
) {
    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;

    if mousebtn.pressed(MouseButton::Right) {
        if kbd.pressed(KeyCode::AltLeft) {
            for ev in ev_motion.read() {
                rotation_move += ev.delta;
            }
        } else {
            for ev in ev_motion.read() {
                pan += ev.delta;
            }
        }
    }
    for ev in ev_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Pixel => {
                scroll += ev.y;
            }
            MouseScrollUnit::Line => {
                scroll -= ev.y * 64.0;
            }
        }
    }
    for (mut pan_orbit, mut transform, camera) in q_camera.iter_mut() {
        let mut any = false;
        if scroll != 0.0 {
            any = true;
            pan_orbit.radius += scroll;
        }
        if pan != Vec2::ZERO {
            // pan.x = -pan.x;
            // let mat = Mat2::from_angle(pan_orbit.yaw);
            // pan_orbit.center += mat.mul_vec2(pan);
            if let Some(cursor_position) = q_window.get_single().ok().and_then(|window| window.cursor_position()) {
                let cursor_position_old = cursor_position - pan;
                let camera_transform = GlobalTransform::from(*transform);
                let groundpos_new = compute_cursor_to_ground_plane(cursor_position, &camera_transform, camera);
                let groundpos_old = compute_cursor_to_ground_plane(cursor_position_old, &camera_transform, camera);
                if let (Some(groundpos_new), Some(groundpos_old)) = (groundpos_new, groundpos_old) {
                    any = true;
                    pan_orbit.center -= groundpos_new - groundpos_old;
                }
            };
        }
        if rotation_move != Vec2::ZERO {
            any = true;
            pan_orbit.yaw -= rotation_move.x * 0.01;
            pan_orbit.pitch += rotation_move.y * 0.01;
        }
        if any || !*init {
            *init = true;
            transform.rotation = Quat::from_euler(EulerRot::YXZ, pan_orbit.yaw, -pan_orbit.pitch, 0.0);
            let v = -transform.rotation.mul_vec3(Vec3::NEG_Z);
            transform.translation = Vec3::new(pan_orbit.center.x, 0.0, -pan_orbit.center.y)
                + v * pan_orbit.radius;
        }
    }
    ev_motion.clear();
}

fn cursor_to_ground_plane(
    mut wc: ResMut<WorldCursor>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
    let Ok((camera, camera_transform)) = q_camera.get_single() else {
        return;
    };
    let Some(cursor_position) = q_window.get_single().ok().and_then(|window| window.cursor_position()) else {
        return;
    };

    let Some(newpos) = compute_cursor_to_ground_plane(cursor_position, camera_transform, camera) else {
        return;
    };

    if newpos == wc.pos && newpos == wc.pos_prev {
        return;
    }

    wc.pos_prev = wc.pos;
    wc.pos = newpos;
}

fn compute_cursor_to_ground_plane(
    cursor_position: Vec2,
    camera_transform: &GlobalTransform,
    camera: &Camera,
) -> Option<Vec2> {
    let plane_origin = Vec3::ZERO;
    let plane = Plane3d::new(Vec3::Y);

    let ray = camera.viewport_to_world(camera_transform, cursor_position)?;
    let distance = ray.intersect_plane(plane_origin, plane)?;
    let global_cursor = ray.get_point(distance);

    Some(Vec2::new(global_cursor.x, -global_cursor.z))
}

fn grid_cursor<C: Coord>(
    crs_in: Res<WorldCursor>,
    mut crs_out: ResMut<GridCursor>,
    mapdesc: Res<MapDescriptor>,
    index: Option<Res<MapTileIndex<C>>>,
    mut cursor_tile: ResMut<GridCursorTileEntity>,
) {
    match C::TOPOLOGY {
        Topology::Hex => {
            let tdim = Vec2::new((3f64.sqrt() * TILE_SCALE as f64 / 2.0) as f32, TILE_SCALE);
            let conv = bevy::math::Mat2::from_cols_array(
                &[tdim.x, 0.0, tdim.x * 0.5, tdim.y * 0.75]
            ).inverse();
            let adj = conv * crs_in.pos;
            let new = Hex::from_f32_clamped(adj.into());
            if new.ring() <= mapdesc.size {
                let new_pos = Pos::from(new);
                if crs_out.0 != new_pos {
                    crs_out.0 = new_pos;
                    let new_e = index.and_then(|inner| inner.0.get(new_pos.into()).cloned());
                    if cursor_tile.0 != new_e {
                        cursor_tile.0 = new_e;
                    }
                }
            }
        }
        Topology::Sq => {
            unimplemented!()
        }
    };
}
