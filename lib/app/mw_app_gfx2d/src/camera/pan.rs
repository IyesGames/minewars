use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, window::PrimaryWindow};
use mw_app_core::{camera::{input::*, *}, input::*};

use crate::{prelude::*, settings::Camera2dControlSettings};

use super::CameraJumpTweenState;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        pan_tween
            .run_if(rc_pan_tween)
            .in_set(SetStage::Provide(CameraControlSS)),
    ));
    // inputs
    app.add_systems(
        InputAnalogOnStart(ANALOG_PAN.into()),
        input_analog_pan_start
    );
    app.add_systems(
        InputAnalogOnStop(ANALOG_PAN.into()),
        input_analog_pan_stop
    );
    app.add_systems(Update, (
        input_pan_edge
            // edge panning only when no motion controls are active
            .run_if(none_filter::<(With<CameraControlInput>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
        input_analog_pan_motion
            .in_set(OnMouseMotionEventSet)
            .run_if(any_filter::<(With<AnalogPan>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
        input_analog_pan_scroll
            .in_set(OnMouseScrollEventSet)
            .run_if(any_filter::<(With<AnalogPan>, With<InputAnalogActive>, With<AnalogSourceMouseScroll>)>),
    )
        .in_set(GameInputSet)
        .in_set(SetStage::Provide(CameraControlSS))
        .in_set(SetStage::Want(GameInputSS::Handle))
    );
}

#[derive(Component, Default)]
pub struct CameraPanState {
    start_cursor: Option<Vec2>,
    start_translation: Vec2,
    tween_timer: Option<Timer>,
    curve: CubicSegment<Vec2>,
    tween_start_translation: Vec2,
    tween_target_translation: Vec2,
}

fn input_analog_pan_start(
    settings: Settings,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraPanState, &Transform, &mut CameraJumpTweenState,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let window = q_window.single();
    for (mut pan, xf, mut jump) in &mut q_camera {
        pan.start_cursor = window.cursor_position();
        pan.start_translation = xf.translation.truncate();
        pan.tween_timer = None;
        pan.curve = CubicSegment::new_bezier(
            s_input.pan_tween_curve.0,
            s_input.pan_tween_curve.1,
        );
        jump.timer = None;
    }
}

fn input_analog_pan_stop( 
    mut q_camera: Query<(
        &mut CameraPanState, &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    for (mut pan, mut xf, proj) in &mut q_camera {
        pan.start_cursor = None;
        // round the camera position to the nearest multiple
        // of its scale, to ensure graphics don't get blurry
        xf.translation.x = (xf.translation.x / proj.scale).round() * proj.scale;
        xf.translation.y = (xf.translation.y / proj.scale).round() * proj.scale;
    }
}

fn input_analog_pan_motion(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraPanState, &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    let window = q_window.single();
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    for (mut pan, mut xf, proj) in &mut q_camera {
        let Some(start_cursor) = pan.start_cursor else {
            continue;
        };
        let mut delta = cursor - start_cursor;
        delta.y = -delta.y;
        let delta = (xf.rotation * delta.extend(0.0)).truncate() * proj.scale;
        xf.translation.x = pan.start_translation.x - delta.x;
        xf.translation.y = pan.start_translation.y - delta.y;
        pan.tween_timer = None;
        pan.tween_target_translation = xf.translation.truncate();
    }
}

fn input_analog_pan_scroll(
    settings: Settings,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &mut CameraPanState, &Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let mut delta = Vec2::ZERO;
    for ev in evr_scroll.read() {
        delta += match ev.unit {
            MouseScrollUnit::Line => {
                let (mut x, mut y) = (ev.x, ev.y);
                if !s_input.scroll_pan_allow_fractional_lines {
                    if x < 0.0 {
                        x = x.floor();
                    }
                    if y < 0.0 {
                        y = y.floor();
                    }
                    if x > 0.0 {
                        x = x.ceil();
                    }
                    if y > 0.0 {
                        y = y.ceil();
                    }
                }
                Vec2::new(
                    x * s_input.scroll_pan_per_line,
                    y * s_input.scroll_pan_per_line,
                )
            }
            MouseScrollUnit::Pixel => {
                Vec2::new(
                    ev.x * s_input.scroll_pan_per_pixel,
                    ev.y * s_input.scroll_pan_per_pixel,
                )
            }
        };
    }
    if delta == Vec2::ZERO {
        return;
    }
    delta.y = -delta.y;
    for (mut pan, xf, proj) in &mut q_camera {
        let delta = (xf.rotation * delta.extend(0.0)).truncate() * proj.scale;
        pan.tween_target_translation.x -= delta.x;
        pan.tween_target_translation.y -= delta.y;
        pan.start_translation.x -= delta.x;
        pan.start_translation.y -= delta.y;
        pan.tween_start_translation = xf.translation.truncate();
        pan.tween_timer = Some(Timer::new(Duration::from_secs_f32(s_input.pan_tween_duration), TimerMode::Once));
    }
}

fn rc_pan_tween(
    q_camera: Query<&CameraPanState, With<ActiveGameCamera>>,
) -> bool {
    q_camera.iter().any(|pan| pan.tween_timer.is_some())
}

fn pan_tween(
    time: Res<Time>,
    mut q_camera: Query<(
        &mut CameraPanState, &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    for (mut pan, mut xf, proj) in &mut q_camera {
        let pan = &mut *pan;
        let Some(timer) = &mut pan.tween_timer else {
            continue;
        };
        if timer.finished() {
            pan.tween_timer = None;
            xf.translation.x = pan.tween_target_translation.x;
            xf.translation.y = pan.tween_target_translation.y;
            xf.translation.x = (xf.translation.x / proj.scale).round() * proj.scale;
            xf.translation.y = (xf.translation.y / proj.scale).round() * proj.scale;
            continue;
        }
        let fraction = timer.fraction();
        let t = pan.curve.ease(fraction);
        let translation = pan.tween_start_translation.lerp(pan.tween_target_translation, t);
        xf.translation.x = translation.x;
        xf.translation.y = translation.y;
        timer.tick(time.delta());
    }
}

fn input_pan_edge(
    settings: Settings,
    time: Res<Time>,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
    mut needs_rounding: Local<bool>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let Ok(window) = q_window.get_single() else {
        return;
    };
    let mut dir = Vec2::ZERO;
    if let Some(cursor) = window.cursor_position() {
        if cursor.x < s_input.edge_pan_margin {
            dir.x -= 1.0;
        }
        if cursor.x >= window.width() - s_input.edge_pan_margin {
            dir.x += 1.0;
        }
        if cursor.y < s_input.edge_pan_margin {
            dir.y += 1.0;
        }
        if cursor.y >= window.height() - s_input.edge_pan_margin {
            dir.y -= 1.0;
        }
        dir = dir.normalize_or_zero();
    };
    if dir != Vec2::ZERO {
        *needs_rounding = true;
        for (mut xf, proj) in &mut q_camera {
            let dir = (xf.rotation * dir.extend(0.0)).truncate();
            let delta = dir * s_input.edge_pan_speed * proj.scale * time.delta_seconds();
            xf.translation.x += delta.x;
            xf.translation.y += delta.y;
        }
    } else if *needs_rounding {
        *needs_rounding = false;
        for (mut xf, proj) in &mut q_camera {
            xf.translation.x = (xf.translation.x / proj.scale).round() * proj.scale;
            xf.translation.y = (xf.translation.y / proj.scale).round() * proj.scale;
        }
    }
}
