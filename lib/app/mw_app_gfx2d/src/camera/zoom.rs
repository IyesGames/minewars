use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, window::PrimaryWindow};
use mw_app_core::{camera::{input::*, *}, input::*};

use crate::{prelude::*, settings::Camera2dControlSettings};

use super::CameraJumpTweenState;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, (
        zoom_tween
            .run_if(rc_zoom_tween)
            .in_set(SetStage::Provide(CameraControlSS)),
    ));
    app.add_systems(
        InputAnalogOnStart(ANALOG_ZOOM.into()),
        input_analog_zoom_start
    );
    app.add_systems(
        InputAnalogOnStop(ANALOG_ZOOM.into()),
        input_analog_zoom_stop
    );
    app.add_systems(Update, (
        input_analog_zoom_motion
            .in_set(OnMouseMotionEventSet)
            .run_if(any_filter::<(With<AnalogZoom>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
        input_analog_zoom_scroll
            .in_set(OnMouseScrollEventSet)
            .run_if(any_filter::<(With<AnalogZoom>, With<InputAnalogActive>, With<AnalogSourceMouseScroll>)>),
    )
        .in_set(GameInputSet)
        .in_set(SetStage::Provide(CameraControlSS))
        .in_set(SetStage::Want(GameInputSS::Handle))
    );
}

#[derive(Component, Default)]
pub struct CameraZoomState {
    start_cursor: Option<Vec2>,
    start_scale: f32,
    snap_break_accum: f32,
    tween_timer: Option<Timer>,
    curve: CubicSegment<Vec2>,
    tween_start_scale: f32,
    tween_target_scale: f32,
}

fn input_analog_zoom_start(
    settings: Settings,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraZoomState, &mut CameraJumpTweenState, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let window = q_window.single();
    for (mut zoom, mut jump, proj) in &mut q_camera {
        zoom.start_cursor = window.cursor_position();
        zoom.start_scale = proj.scale;
        zoom.snap_break_accum = 1.0;
        zoom.tween_timer = None;
        zoom.curve = CubicSegment::new_bezier(
            s_input.zoom_tween_curve.0,
            s_input.zoom_tween_curve.1,
        );
        jump.timer = None;
    }
}

fn input_analog_zoom_stop(
    mut q_camera: Query<(
        &mut CameraZoomState, &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    for (mut zoom, mut xf, proj) in &mut q_camera {
        zoom.start_cursor = None;
        xf.translation.x = (xf.translation.x / proj.scale).round() * proj.scale;
        xf.translation.y = (xf.translation.y / proj.scale).round() * proj.scale;
    }
}

fn input_analog_zoom_motion(
    settings: Settings,
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraZoomState, &mut OrthographicProjection,
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
    for (mut zoom, mut proj) in &mut q_camera {
        let Some(start_cursor) = zoom.start_cursor else {
            continue;
        };
        let d_start = center.distance(start_cursor);
        let d_cur = center.distance(cursor);
        if d_start == 0.0 || d_cur == 0.0 {
            continue;
        }
        let ratio = d_start / d_cur;
        proj.scale = (zoom.start_scale * ratio)
            .clamp(s_input.zoom_min, s_input.zoom_max);
        if s_input.enable_zoom_motion_snapping {
            let nearest_pow2 = proj.scale.log2().round().exp2().round();
            let (t_lo, t_hi) = if s_input.zoom_level_snap_threshold > 0.0 && s_input.zoom_level_snap_threshold < 1.0 {
                (s_input.zoom_level_snap_threshold, 1.0 / s_input.zoom_level_snap_threshold)
            } else if s_input.zoom_level_snap_threshold > 1.0 && s_input.zoom_level_snap_threshold < 2.0 {
                (1.0 / s_input.zoom_level_snap_threshold, s_input.zoom_level_snap_threshold)
            } else {
                (1.0, 1.0)
            };
            if proj.scale < nearest_pow2 * t_hi && proj.scale > nearest_pow2 * t_lo {
                proj.scale = nearest_pow2;
            }
        }
        zoom.tween_timer = None;
        zoom.tween_target_scale = proj.scale;
    }
}

fn input_analog_zoom_scroll(
    settings: Settings,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &mut CameraZoomState, &mut OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let mut total_lines = 0.0;
    let mut total_pixels = 0.0;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                total_lines -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_pixels -= ev.y;
            }
        }
    }
    if total_pixels != 0.0 {
        let total_zoom = (total_pixels * s_input.scroll_zoom_per_pixel).exp();
        for (mut zoom, mut proj) in &mut q_camera {
            zoom.start_scale = (zoom.start_scale * total_zoom)
                .clamp(s_input.zoom_min, s_input.zoom_max);
            proj.scale = (proj.scale * total_zoom)
                .clamp(s_input.zoom_min, s_input.zoom_max);
            zoom.tween_timer = None;
            zoom.tween_target_scale = proj.scale;
            if s_input.enable_zoom_scroll_pixel_snapping {
                let nearest_pow2 = proj.scale.log2().round().exp2().round();
                let (t_lo, t_hi) = if s_input.zoom_level_snap_threshold > 0.0 && s_input.zoom_level_snap_threshold < 1.0 {
                    (s_input.zoom_level_snap_threshold, 1.0 / s_input.zoom_level_snap_threshold)
                } else if s_input.zoom_level_snap_threshold > 1.0 && s_input.zoom_level_snap_threshold < 2.0 {
                    (1.0 / s_input.zoom_level_snap_threshold, s_input.zoom_level_snap_threshold)
                } else {
                    (1.0, 1.0)
                };
                if proj.scale < nearest_pow2 * t_hi && proj.scale > nearest_pow2 * t_lo {
                    zoom.start_scale = nearest_pow2;
                    proj.scale = nearest_pow2;
                    zoom.snap_break_accum *= total_zoom;
                    if zoom.snap_break_accum > t_hi || zoom.snap_break_accum < t_lo {
                        zoom.start_scale = zoom.start_scale * zoom.snap_break_accum;
                        proj.scale = (proj.scale * zoom.snap_break_accum)
                            .clamp(s_input.zoom_min, s_input.zoom_max);
                        zoom.snap_break_accum = 1.0;
                    }
                } else {
                    zoom.snap_break_accum = 1.0;
                }
            }
        }
    }
    if total_lines != 0.0 {
        if !s_input.scroll_zoom_allow_fractional_lines {
            if total_lines < 0.0 {
                total_lines = total_lines.floor();
            }
            if total_lines > 0.0 {
                total_lines = total_lines.ceil();
            }
        }
        let total_zoom = (total_lines * s_input.scroll_zoom_per_line).exp();
        for (mut zoom, proj) in &mut q_camera {
            zoom.start_scale = zoom.start_scale * total_zoom
                .clamp(s_input.zoom_min, s_input.zoom_max);
            zoom.tween_timer = Some(Timer::new(Duration::from_secs_f32(s_input.zoom_tween_duration), TimerMode::Once));
            zoom.tween_start_scale = proj.scale;
            let old = zoom.tween_target_scale;
            zoom.tween_target_scale = (zoom.tween_target_scale * total_zoom)
                .clamp(s_input.zoom_min, s_input.zoom_max);
            if s_input.enable_zoom_scroll_line_snapping {
                let nearest_pow2 = if zoom.tween_target_scale > old {
                    zoom.tween_target_scale.log2().floor().exp2().floor()
                } else {
                    zoom.tween_target_scale.log2().ceil().exp2().ceil()
                };
                let (t_lo, t_hi) = if s_input.zoom_level_snap_threshold > 0.0 && s_input.zoom_level_snap_threshold < 1.0 {
                    (s_input.zoom_level_snap_threshold, 1.0 / s_input.zoom_level_snap_threshold)
                } else if s_input.zoom_level_snap_threshold > 1.0 && s_input.zoom_level_snap_threshold < 2.0 {
                    (1.0 / s_input.zoom_level_snap_threshold, s_input.zoom_level_snap_threshold)
                } else {
                    (1.0, 1.0)
                };
                if zoom.tween_target_scale < nearest_pow2 * t_hi && zoom.tween_target_scale > nearest_pow2 * t_lo {
                    zoom.start_scale = nearest_pow2;
                    zoom.tween_target_scale = nearest_pow2;
                    zoom.snap_break_accum *= total_zoom;
                    if zoom.snap_break_accum > t_hi || zoom.snap_break_accum < t_lo {
                        zoom.start_scale = zoom.start_scale * zoom.snap_break_accum;
                        zoom.tween_target_scale = (zoom.tween_target_scale * zoom.snap_break_accum)
                            .clamp(s_input.zoom_min, s_input.zoom_max);
                        zoom.snap_break_accum = 1.0;
                    }
                } else {
                    zoom.snap_break_accum = 1.0;
                }
            }
        }
    }
}

fn rc_zoom_tween(
    q_camera: Query<&CameraZoomState, With<ActiveGameCamera>>,
) -> bool {
    q_camera.iter().any(|zoom| zoom.tween_timer.is_some())
}

fn zoom_tween(
    time: Res<Time>,
    mut q_camera: Query<(
        &mut CameraZoomState, &mut Transform, &mut OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    for (mut zoom, mut xf, mut proj) in &mut q_camera {
        let zoom = &mut *zoom;
        let Some(timer) = &mut zoom.tween_timer else {
            continue;
        };
        if timer.finished() {
            zoom.tween_timer = None;
            proj.scale = zoom.tween_target_scale;
            xf.translation.x = (xf.translation.x / proj.scale).round() * proj.scale;
            xf.translation.y = (xf.translation.y / proj.scale).round() * proj.scale;
            continue;
        }
        let fraction = timer.fraction();
        let t = zoom.curve.ease(fraction);
        proj.scale = zoom.tween_start_scale.lerp(zoom.tween_target_scale, t);
        timer.tick(time.delta());
    }
}
