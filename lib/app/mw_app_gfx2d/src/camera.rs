use mw_app_core::{camera::{input::*, *}, graphics::{Gfx2dEnabled, GraphicsGovernor}, input::*};

use crate::prelude::*;

mod cursor;
mod jump;
mod pan;
mod rotate;
mod zoom;

use jump::CameraJumpTweenState;
use pan::CameraPanState;
use rotate::CameraRotateState;
use zoom::CameraZoomState;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_game_camera
            .run_if(any_filter::<(With<GraphicsGovernor>, With<Gfx2dEnabled>)>)
    );
    app.add_plugins((
        cursor::plugin,
        jump::plugin,
        pan::plugin,
        rotate::plugin,
        zoom::plugin,
    ));
}

#[derive(Bundle, Default)]
struct Active2dCameraBundle {
    camera: Camera2dBundle,
    game: GameCameraBundle,
    active: ActiveGameCamera,
    pan: CameraPanState,
    rotate: CameraRotateState,
    zoom: CameraZoomState,
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
