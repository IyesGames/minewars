use bevy::{input::mouse::{MouseScrollUnit, MouseWheel}, window::PrimaryWindow};
use mw_app_core::{camera::{input::*, *}, graphics::{Gfx2dEnabled, GraphicsGovernor}, input::*, map::*};

use crate::{prelude::*, settings::Camera2dControlSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_game_camera
            .run_if(any_filter::<(With<GraphicsGovernor>, With<Gfx2dEnabled>)>)
    );
    app.add_systems(Update, (
        jump_tween_start
            .in_set(SetStage::WantChanged(CameraJumpSS)),
        jump_tween
            .run_if(rc_jump_tween)
            .in_set(SetStage::Provide(CameraControlSS)),
    ).chain());
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
        input_pan_edge
            // edge panning only when no other panning is active
            .run_if(any_filter::<(With<AnalogPan>, Without<InputAnalogActive>)>),
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
        input_analog_grid_cursor_motion
            .in_set(OnMouseMotionEventSet)
            .run_if(any_filter::<(With<AnalogGridCursor>, With<InputAnalogActive>, With<AnalogSourceMouseMotion>)>),
    )
        .in_set(GameInputSet)
        .in_set(SetStage::Provide(CameraControlSS))
        .in_set(SetStage::Want(GameInputSS::Handle))
    );
}

#[derive(Bundle, Default)]
struct Active2dCameraBundle {
    camera: Camera2dBundle,
    game: GameCameraBundle,
    active: ActiveGameCamera,
    pan: CameraPanState,
    rotate: CameraRotateState,
    jump: CameraJumpTweenState,
}

fn setup_game_camera(
    mut commands: Commands,
    q_actions: Query<Entity, (With<CameraInput>, With<InputAction>)>,
    q_analogs: Query<Entity, (With<CameraInput>, With<InputAnalog>)>,
) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scale = 8.0;
    commands.spawn(Active2dCameraBundle {
        camera,
        ..Default::default()
    });

    for e in &q_actions {
        commands.entity(e).insert(InputActionEnabled);
    }
    for e in &q_analogs {
        commands.entity(e).insert(InputAnalogEnabled);
    }
}

#[derive(Component, Default)]
struct CameraJumpTweenState {
    timer: Option<Timer>,
    curve: CubicSegment<Vec2>,
    start_translation: Vec2,
    end_translation: Vec2,
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

fn input_analog_grid_cursor_motion(
    mut q_map: Query<(
        &mut GridCursor, &mut GridCursorTileEntity, &mut GridCursorTileTranslation,
        &MapDescriptor, &MapTileIndex,
    ), With<MapGovernor>>,
    q_camera: Query<(
        &Transform, &Camera,
    ), With<ActiveGameCamera>>,
    q_window: Query<&Window, With<PrimaryWindow>>,
) {
    let (mut crs, mut gcte, mut gctt, desc, index) = q_map.single_mut();
    let Ok((xf_camera, camera)) = q_camera.get_single() else {
        return;
    };
    let Ok(window) = q_window.get_single() else {
        return;
    };
    // assuming camera not affected by hierarchy
    let gxf_camera = GlobalTransform::from(*xf_camera);
    let Some(cursor) = window.cursor_position()
        .and_then(|pos| camera.viewport_to_world(&gxf_camera, pos))
        .map(|ray| ray.origin.truncate())
    else {
        crs.0 = None;
        gcte.0 = None;
        gctt.0 = None;
        return;
    };
    match desc.topology {
        Topology::Hex => {
            let tdim = Vec2::new(crate::misc::sprite::WIDTH6, crate::misc::sprite::HEIGHT6);
            let conv = bevy::math::Mat2::from_cols_array(
                &[tdim.x, 0.0, tdim.x * 0.5, tdim.y * 0.75]
            ).inverse();
            let adj = conv * cursor;
            let new = Hex::from_f32_clamped(adj.into());
            if new.ring() <= desc.size {
                let new_pos = Pos::from(new);
                if crs.0 != Some(new_pos) {
                    crs.0 = Some(new_pos);
                    gctt.0 = Some((new.translation() * tdim).extend(0.0));
                    if let Some(&new_e) = index.0.get(new_pos) {
                        gcte.0 = Some(new_e);
                    }
                }
            } else {
                crs.0 = None;
                gcte.0 = None;
                gctt.0 = None;
            }
        }
        Topology::Sq => {
            let tdim = Vec2::new(crate::misc::sprite::WIDTH4, crate::misc::sprite::HEIGHT4);
            let adj = cursor / tdim;
            let new = Sq::from_f32_clamped(adj.into());
            if new.ring() <= desc.size {
                let new_pos = Pos::from(new);
                if crs.0 != Some(new_pos) {
                    crs.0 = Some(new_pos);
                    gctt.0 = Some((new.translation() * tdim).extend(0.0));
                    if let Some(&new_e) = index.0.get(new_pos) {
                        gcte.0 = Some(new_e);
                    }
                }
            } else {
                crs.0 = None;
                gcte.0 = None;
                gctt.0 = None;
            }
        }
    }
}

fn jump_tween_start(
    settings: Settings,
    mut evr_jump: EventReader<CameraJumpTo>,
    q_map: Query<&MapDescriptor, With<MapGovernor>>,
    mut q_camera: Query<(
        &mut CameraJumpTweenState, &Transform,
    ), With<ActiveGameCamera>>,
) {
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
    let desc = q_map.single();
    if let Some(ev) = evr_jump.read().last() {
        let pos_translation = match desc.topology {
            Topology::Hex => {
                let tdim = Vec2::new(crate::misc::sprite::WIDTH6, crate::misc::sprite::HEIGHT6);
                Hex::from(ev.0).translation() * tdim
            },
            Topology::Sq => {
                let tdim = Vec2::new(crate::misc::sprite::WIDTH4, crate::misc::sprite::HEIGHT4);
                Sq::from(ev.0).translation() * tdim
            },
        };
        for (mut tween, xf) in &mut q_camera {
            tween.start_translation = xf.translation.truncate();
            tween.end_translation = pos_translation;
            tween.timer = Some(Timer::new(
                Duration::from_secs_f32(s_input.jump_tween_duration),
                TimerMode::Once
            ));
            tween.curve = CubicSegment::new_bezier(
                s_input.jump_tween_curve.0,
                s_input.jump_tween_curve.1,
            );
        }
    }
}

fn rc_jump_tween(
    q_camera: Query<&CameraJumpTweenState, With<ActiveGameCamera>>,
) -> bool {
    q_camera.iter().any(|tween| tween.timer.is_some())
}

fn jump_tween(
    time: Res<Time>,
    mut q_camera: Query<(
        &mut CameraJumpTweenState, &mut Transform,
    ), With<ActiveGameCamera>>,
) {
    for (mut tween, mut xf) in &mut q_camera {
        let tween = &mut *tween;
        let Some(timer) = &mut tween.timer else {
            continue;
        };
        if timer.finished() {
            tween.timer = None;
            continue;
        }
        let fraction = timer.fraction();
        let t = tween.curve.ease(fraction);
        let translation = tween.start_translation.lerp(tween.end_translation, t);
        xf.translation.x = translation.x;
        xf.translation.y = translation.y;
        timer.tick(time.delta());
    }
}

fn input_analog_pan_start(
    q_window: Query<&Window, With<PrimaryWindow>>,
    mut q_camera: Query<(
        &mut CameraPanState, &Transform, &mut CameraJumpTweenState,
    ), With<ActiveGameCamera>>,
) {
    let window = q_window.single();
    for (mut pan, xf, mut jump) in &mut q_camera {
        pan.start_cursor = window.cursor_position();
        pan.start_translation = xf.translation.truncate();
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
        let mut delta = cursor - start_cursor;
        delta.y = -delta.y;
        let delta = (xf.rotation * delta.extend(0.0)).truncate() * proj.scale;
        xf.translation.x = pan.start_translation.x - delta.x;
        xf.translation.y = pan.start_translation.y - delta.y;
    }
}

fn input_analog_pan_scroll(
    settings: Settings,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(
        &mut CameraPanState, &mut Transform, &OrthographicProjection,
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
    for (mut pan, mut xf, proj) in &mut q_camera {
        let delta = (xf.rotation * delta.extend(0.0)).truncate() * proj.scale;
        xf.translation.x -= delta.x;
        xf.translation.y -= delta.y;
        pan.start_translation.x -= delta.x;
        pan.start_translation.y -= delta.y;
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
    let s_input = settings.get::<Camera2dControlSettings>().unwrap();
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
