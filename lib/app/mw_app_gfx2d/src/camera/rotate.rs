use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, window::PrimaryWindow};
use mw_app_core::{camera::{input::*, *}, input::*, map::*};

use crate::{prelude::*, settings::Camera2dControlSettings};

use super::CameraJumpTweenState;

pub fn plugin(app: &mut App) {
    // inputs
    app.add_systems(
        InputAnalogOnStart(ANALOG_ROTATE.into()),
        input_analog_rotate_start
    );
    app.add_systems(
        InputAnalogOnStop(ANALOG_ROTATE.into()),
        input_analog_rotate_stop
    );
    app.add_systems(Update, (
        input_analog_rotate_motion
            .in_set(OnMouseMotionEventSet)
            .run_if(any_filter::<(With<AnalogRotate>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
        input_analog_rotate_scroll
            .in_set(OnMouseScrollEventSet)
            .run_if(any_filter::<(With<AnalogRotate>, With<InputAnalogActive>, With<AnalogSourceMouseScroll>)>),
    )
        .in_set(GameInputSet)
        .in_set(SetStage::Provide(CameraControlSS))
        .in_set(SetStage::Want(GameInputSS::Handle))
    );
}

#[derive(Component, Default)]
pub struct CameraRotateState {
    start_cursor: Option<Vec2>,
    angle: f32,
    angle_start: f32,
    snap_break_accum: f32,
}

fn input_analog_rotate_start(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraRotateState, &mut CameraJumpTweenState,
    ), With<ActiveGameCamera>>,
) {
    let window = q_window.single();
    for (mut rotate, mut jump) in &mut q_camera {
        rotate.start_cursor = window.cursor_position();
        rotate.angle_start = rotate.angle;
        rotate.snap_break_accum = 0.0;
        jump.timer = None;
    }
}

fn input_analog_rotate_stop(
    mut q_camera: Query<(
        &mut CameraRotateState,
    ), With<ActiveGameCamera>>,
) {
    for (mut rotate,) in &mut q_camera {
        rotate.start_cursor = None;
    }
}

fn input_analog_rotate_motion(
    settings: Settings,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_map: Query<&MapDescriptor, With<MapGovernor>>,
    mut q_camera: Query<(
        &mut CameraRotateState, &mut Transform,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let window = q_window.single();
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    let center = Vec2::new(
        window.width() / 2.0,
        window.height() / 2.0,
    );
    let desc = q_map.single();
    for (mut rotate, mut xf) in &mut q_camera {
        let Some(start_cursor) = rotate.start_cursor else {
            continue;
        };
        let d_start = (start_cursor - center).normalize_or_zero();
        let d_cur = (cursor - center).normalize_or_zero();
        if d_start == Vec2::ZERO || d_cur == Vec2::ZERO {
            continue;
        }
        let angle_between = d_start.angle_between(d_cur);
        rotate.angle = rotate.angle_start + angle_between;
        if s_input.enable_rotate_motion_snapping {
            let snap_interval = match desc.topology {
                Topology::Hex => s_input.rotate_hex_snap_interval,
                Topology::Sq => s_input.rotate_sq_snap_interval,
            }.to_radians();
            let snap_angle = (rotate.angle / snap_interval).round() * snap_interval;
            let snap_delta = (rotate.angle - snap_angle).abs();
            if snap_delta < s_input.rotate_snap_threshold.to_radians() {
                rotate.angle = snap_angle;
            }
        }
        xf.rotation = Quat::from_rotation_z(rotate.angle);
    }
}

fn input_analog_rotate_scroll(
    settings: Settings,
    mut evr_scroll: EventReader<MouseWheel>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    q_map: Query<&MapDescriptor, With<MapGovernor>>,
    mut q_camera: Query<(
        &mut CameraRotateState, &mut Transform,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let desc = q_map.single();
    let mut delta = 0.0;
    let mut enable_snapping = false;
    for ev in evr_scroll.read() {
        delta += match ev.unit {
            MouseScrollUnit::Line => {
                let mut y = ev.y;
                if !s_input.scroll_rotate_allow_fractional_lines {
                    if y < 0.0 {
                        y = y.floor();
                    }
                    if y > 0.0 {
                        y = y.ceil();
                    }
                }
                enable_snapping |= s_input.enable_rotate_scroll_line_snapping;
                y * s_input.scroll_rotate_per_line
            }
            MouseScrollUnit::Pixel => {
                enable_snapping |= s_input.enable_rotate_scroll_pixel_snapping;
                ev.y * s_input.scroll_rotate_per_pixel
            }
        };
    }
    if delta == 0.0 {
        return;
    }
    delta = delta.to_radians();
    if s_input.scroll_rotate_invert_leftside {
        let window = q_window.single();
        if let Some(cursor) = window.cursor_position() {
            if cursor.x < window.width() / 2.0 {
                delta = -delta;
            }
        }
    }
    for (mut rotate, mut xf) in &mut q_camera {
        rotate.angle_start += delta;
        rotate.angle += delta;
        if enable_snapping {
            let snap_interval = match desc.topology {
                Topology::Hex => s_input.rotate_hex_snap_interval,
                Topology::Sq => s_input.rotate_sq_snap_interval,
            }.to_radians();
            let snap_angle = (rotate.angle / snap_interval).round() * snap_interval;
            let snap_delta = (rotate.angle - snap_angle).abs();
            let threshold = s_input.rotate_snap_threshold.to_radians();
            if snap_delta < threshold {
                rotate.angle_start = snap_angle;
                rotate.angle = snap_angle;
                rotate.snap_break_accum += delta;
                if rotate.snap_break_accum.abs() > threshold {
                    rotate.angle_start += rotate.snap_break_accum;
                    rotate.angle += rotate.snap_break_accum;
                    rotate.snap_break_accum = 0.0;
                }
            } else {
                rotate.snap_break_accum = 0.0;
            }
        }
        xf.rotation = Quat::from_rotation_z(rotate.angle);
    }
}
