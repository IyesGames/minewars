use bevy::{render::camera::ScalingMode, window::PrimaryWindow};
use mw_app::camera::{GameCamera, GridCursor, GridCursorSet};
use mw_common::{game::MapDescriptor, grid::*};

use crate::{prelude::*, ui::UiCamera};
use super::*;

pub struct Gfx3dCameraPlugin;

impl Plugin for Gfx3dCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_game_camera_3d.in_set(Gfx3dSet::Any));
        app.add_systems(Update, (
            cursor_to_ground_plane
                .in_set(InGameSet(None))
                .in_set(Gfx3dSet::Any)
                .in_set(WorldCursorSet),
            grid_cursor
                .in_set(GridCursorSet)
                .in_set(Gfx3dSet::Any)
                .after(WorldCursorSet)
        ));
    }
}

fn setup_game_camera_3d(
    mut commands: Commands,
) {
    let projection = OrthographicProjection {
        near: 0.0,
        far: RENDER_RANGE,
        scaling_mode: ScalingMode::FixedVertical(TILE_SCALE),
        scale: 24.0,
        ..Default::default()
    };
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
            projection: projection.into(),
            transform,
            ..Default::default()
        },
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

fn cursor_to_ground_plane(
    mut wc: ResMut<WorldCursor>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_camera: Query<(&Camera, &GlobalTransform), With<GameCamera>>,
) {
    let (camera, camera_transform) = q_camera.single();
    let window = q_window.single();

    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

    let plane_origin = Vec3::ZERO;
    let plane_normal = Vec3::Y;

    let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    let Some(distance) = ray.intersect_plane(plane_origin, plane_normal) else {
        return;
    };

    let global_cursor = ray.get_point(distance);
    wc.pos_prev = wc.pos;
    wc.pos = Vec2::new(global_cursor.x, -global_cursor.z);
}


fn grid_cursor(
    crs_in: Res<WorldCursor>,
    mut crs_out: ResMut<GridCursor>,
    mapdesc: Res<MapDescriptor>,
) {
    match mapdesc.topology {
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
                }
            }
        }
        Topology::Sq => {
            unimplemented!()
        }
    };
}
