use bevy::{input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel}, render::camera::RenderTarget, window::PrimaryWindow};
use mw_app_core::{camera::{input::{AnalogPan, AnalogRotate, ANALOG_PAN, ANALOG_ROTATE}, ActiveGameCamera, CameraInput, GameCamera, GameCameraBundle}, graphics::{Gfx2dEnabled, GraphicsGovernor}, input::*, map::{MapDescriptor, MapGovernor}};

use crate::{prelude::*, settings::Camera2dInputSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_game_camera
            .run_if(any_filter::<(With<GraphicsGovernor>, With<Gfx2dEnabled>)>)
    );
    // inputs
    app.add_systems(
        InputAnalogOnStart(ANALOG_PAN.into()),
        input_analog_pan_start
    );
    app.add_systems(
        InputAnalogOnStop(ANALOG_PAN.into()),
        input_analog_pan_stop
    );
    app.add_systems(
        InputAnalogOnStart(ANALOG_ROTATE.into()),
        input_analog_rotate_start
    );
    app.add_systems(
        InputAnalogOnStop(ANALOG_ROTATE.into()),
        input_analog_rotate_stop
    );
    app.add_systems(Update, (
        input_analog_pan_motion
            .in_set(OnMouseMotionEventSet)
            .run_if(any_filter::<(With<AnalogPan>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
        input_analog_pan_scroll
            .in_set(OnMouseScrollEventSet)
            .run_if(any_filter::<(With<AnalogPan>, With<InputAnalogActive>, With<AnalogSourceMouseScroll>)>),
        input_analog_rotate_motion
            .in_set(OnMouseMotionEventSet)
            .run_if(any_filter::<(With<AnalogRotate>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
        input_analog_rotate_scroll
            .in_set(OnMouseScrollEventSet)
            .run_if(any_filter::<(With<AnalogRotate>, With<InputAnalogActive>, With<AnalogSourceMouseScroll>)>),
    )
        .in_set(GameInputSet)
        .in_set(SetStage::Want(GameInputSS::Handle))
    );
}

fn setup_game_camera(
    mut commands: Commands,
    q_actions: Query<Entity, (With<CameraInput>, With<InputAction>)>,
    q_analogs: Query<Entity, (With<CameraInput>, With<InputAnalog>)>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 8.0;
    commands.spawn((
        camera,
        GameCameraBundle::default(),
        ActiveGameCamera,
        CameraPanState::default(),
        CameraRotateState::default(),
    ));

    for e in &q_actions {
        commands.entity(e).insert(InputActionEnabled);
    }
    for e in &q_analogs {
        commands.entity(e).insert(InputAnalogEnabled);
    }
}

#[derive(Component, Default)]
struct CameraPanState {
    start_cursor: Option<Vec2>,
    start_translation: Vec2,
}

#[derive(Component, Default)]
struct CameraRotateState {
    start_cursor: Option<Vec2>,
    angle: f32,
    angle_start: f32,
}

fn input_analog_pan_start(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraPanState, &Transform
    ), With<ActiveGameCamera>>,
) {
    let window = q_window.single();
    for (mut pan, xf) in &mut q_camera {
        pan.start_cursor = window.cursor_position();
        pan.start_translation = xf.translation.truncate();
    }
}

fn input_analog_pan_stop( 
    mut q_camera: Query<(
        &mut CameraPanState, &mut Transform, &OrthographicProjection
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
        &CameraPanState, &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    let window = q_window.single();
    let Some(cursor) = window.cursor_position() else {
        return;
    };
    for (pan, mut xf, proj) in &mut q_camera {
        let Some(start_cursor) = pan.start_cursor else {
            continue;
        };
        let delta = (cursor - start_cursor) * proj.scale;
        xf.translation.x = pan.start_translation.x - delta.x;
        xf.translation.y = pan.start_translation.y + delta.y;
    }
}

fn input_analog_pan_scroll(
    settings: Settings,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &mut CameraPanState, &mut Transform, &OrthographicProjection,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dInputSettings>().unwrap();
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
    for (mut pan, mut xf, proj) in &mut q_camera {
        xf.translation.x -= delta.x * proj.scale;
        xf.translation.y -= -delta.y * proj.scale;
        pan.start_translation.x -= delta.x * proj.scale;
        pan.start_translation.y -= -delta.y * proj.scale;
    }
}

fn input_analog_rotate_start(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraRotateState,
    ), With<ActiveGameCamera>>,
) {
    let window = q_window.single();
    for (mut rotate,) in &mut q_camera {
        rotate.start_cursor = window.cursor_position();
        rotate.angle_start = rotate.angle;
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
    let s_input = settings.get::<Camera2dInputSettings>().unwrap();
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
        let snap_interval = match desc.topology {
            Topology::Hex => s_input.rotate_hex_snap_interval,
            Topology::Sq => s_input.rotate_sq_snap_interval,
        }.to_radians();
        let snap_angle = (rotate.angle / snap_interval).round() * snap_interval;
        let snap_delta = (rotate.angle - snap_angle).abs();
        if snap_delta < s_input.rotate_snap_threshold.to_radians() {
            rotate.angle = (rotate.angle / snap_interval).round() * snap_interval;
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
    let s_input = settings.get::<Camera2dInputSettings>().unwrap();
    let desc = q_map.single();
    let mut delta = 0.0;
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
                y * s_input.scroll_rotate_per_line
            }
            MouseScrollUnit::Pixel => {
                ev.y * s_input.scroll_rotate_per_pixel
            }
        };
    }
    if delta == 0.0 {
        return;
    }
    if s_input.scroll_rotate_invert_leftside {
        let window = q_window.single();
        if let Some(cursor) = window.cursor_position() {
            if cursor.x < window.width() / 2.0 {
                delta = -delta;
            }
        }
    }
    for (mut rotate, mut xf) in &mut q_camera {
        rotate.angle += delta;
        rotate.angle_start += delta;
        let snap_interval = match desc.topology {
            Topology::Hex => s_input.rotate_hex_snap_interval,
            Topology::Sq => s_input.rotate_sq_snap_interval,
        }.to_radians();
        let snap_angle = (rotate.angle / snap_interval).round() * snap_interval;
        let snap_delta = (rotate.angle - snap_angle).abs();
        if snap_delta < s_input.rotate_snap_threshold.to_radians() {
            rotate.angle = (rotate.angle / snap_interval).round() * snap_interval;
        }
        xf.rotation = Quat::from_rotation_z(rotate.angle);
    }
}
