use bevy::{input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel}, render::camera::RenderTarget, window::PrimaryWindow};
use mw_app_core::{camera::{input::{AnalogPan, ANALOG_PAN}, ActiveGameCamera, CameraInput, GameCamera, GameCameraBundle}, graphics::{Gfx2dEnabled, GraphicsGovernor}, input::*};

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
    app.add_systems(Update, (
        input_analog_pan_motion
            .run_if(any_filter::<(With<AnalogPan>, With<AnalogSourceMouseMotion>)>),
        input_analog_pan_scroll
            .run_if(any_filter::<(With<AnalogPan>, With<AnalogSourceMouseScroll>)>),
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
    for ev in evr_scroll.read() {
        let mut delta = match ev.unit {
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
        if s_input.scroll_pan_invert_x {
            delta.x = -delta.x;
        }
        if s_input.scroll_pan_invert_y {
            delta.y = -delta.y;
        }
        for (mut pan, mut xf, proj) in &mut q_camera {
            xf.translation.x -= delta.x * proj.scale;
            xf.translation.y -= -delta.y * proj.scale;
            pan.start_translation.x -= delta.x * proj.scale;
            pan.start_translation.y -= -delta.y * proj.scale;
        }
    }
}
