use mw_app_core::{camera::{input::*, *}, graphics::{Gfx3dEnabled, GraphicsGovernor}, input::{InputAction, InputActionEnabled, InputAnalog, InputAnalogEnabled}};

use crate::{prelude::*, settings::Camera3dSettings};

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_game_camera
            .run_if(any_filter::<(With<GraphicsGovernor>, With<Gfx3dEnabled>)>)
    );
}

fn setup_game_camera(
    mut commands: Commands,
    settings: Settings,
    q_actions: Query<Entity, (With<CameraInput>, With<InputAction>)>,
    q_analogs: Query<Entity, (With<CameraInput>, With<InputAnalog>)>,
) {
    let s_cam = settings.get::<Camera3dSettings>().unwrap();

    let mut camera = Camera3dBundle::default();
    camera.transform = Transform::from_xyz(600.0, 800.0, -800.0)
        .looking_at(Vec3::ZERO, Vec3::Y);
    camera.projection = PerspectiveProjection {
        fov: s_cam.fov,
        ..Default::default()
    }.into();
    commands.spawn((
        camera,
        GameCameraBundle::default(),
    ));

    for e in &q_actions {
        commands.entity(e).insert(InputActionEnabled);
    }
    for e in &q_analogs {
        commands.entity(e).insert(InputAnalogEnabled);
    }
}
