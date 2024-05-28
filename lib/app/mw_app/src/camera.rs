use mw_app_core::{camera::{input::ACTION_CENTER, ActiveGameCamera, CameraJumpTo, GameCamera}, graphics::*, input::InputActionOnPress, map::{GridCursor, MapGovernor}, ui::UiCamera};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::Menu),
        setup_menu_camera,
    );
    app.add_systems(Update, (
        switch_graphics_style,
    )
        .in_set(InStateSet(AppState::InGame))
        .run_if(any_filter::<(
            With<GraphicsGovernor>,
            With<Gfx2dEnabled>,
            With<Gfx3dEnabled>,
        )>),
    );
    app.add_systems(
        InputActionOnPress(ACTION_CENTER.into()),
        input_action_center,
    );
}

fn setup_menu_camera(
    mut commands: Commands,
) {
    commands.spawn((
        MenuCleanup,
        UiCamera,
        Camera2dBundle::default(),
    ));
}

fn switch_graphics_style(
    mut commands: Commands,
    q_graphics: Query<(
        &CurrentGraphicsStyle,
    ), (
        With<GraphicsGovernor>,
        Changed<CurrentGraphicsStyle>,
    )>,
    mut q_camera_2d: Query<(
        Entity,
        &mut Camera,
    ), (
        With<Camera2d>,
        Without<Camera3d>,
        With<GameCamera>,
    )>,
    mut q_camera_3d: Query<(
        Entity,
        &mut Camera,
    ), (
        With<Camera3d>,
        Without<Camera2d>,
        With<GameCamera>,
    )>,
    q_cam: Query<(), With<Camera>>,
) {
    let Ok((graphics_style,)) = q_graphics.get_single() else {
        return;
    };
    match graphics_style.0 {
        GraphicsStyle::Gfx2d => {
            let Ok((e, mut cam)) = q_camera_2d.get_single_mut() else {
                return;
            };
            cam.is_active = true;
            cam.order = 1;
            commands.entity(e).insert(ActiveGameCamera);
            let Ok((e, mut cam)) = q_camera_3d.get_single_mut() else {
                return;
            };
            cam.is_active = false;
            cam.order = 0;
            commands.entity(e).remove::<ActiveGameCamera>();
        }
        GraphicsStyle::Gfx3d => {
            let Ok((e, mut cam)) = q_camera_3d.get_single_mut() else {
                return;
            };
            cam.is_active = true;
            cam.order = 1;
            commands.entity(e).insert(ActiveGameCamera);
            let Ok((e, mut cam)) = q_camera_2d.get_single_mut() else {
                return;
            };
            cam.is_active = false;
            cam.order = 0;
            commands.entity(e).remove::<ActiveGameCamera>();
        }
    }
}

fn input_action_center(
    mut evw: EventWriter<CameraJumpTo>,
    q_map: Query<&GridCursor, With<MapGovernor>>,
) {
    let crs = q_map.single();
    if let Some(pos) = crs.0 {
        evw.send(CameraJumpTo(pos));
    }
}
