use mw_common::grid::Pos;

use crate::{prelude::*, ui::UiCamera};

pub fn plugin(app: &mut App) {
    app.add_event::<CameraJumpTo>();
    app.configure_stage_set(
        Update,
        CameraControlSS,
        rc_camera_changed,
    );
}

#[derive(Bundle, Default)]
pub struct GameCameraBundle {
    pub cleanup: GamePartialCleanup,
    pub marker: GameCamera,
    pub uimarker: UiCamera,
}

/// Marker for a camera that displays the game world
#[derive(Component, Default)]
pub struct GameCamera;

/// Marker for game camera that the user controls.
#[derive(Component)]
pub struct ActiveGameCamera;

/// Event to cause a (smooth) jump to a given coordinate position
#[derive(Event)]
pub struct CameraJumpTo(pub Pos);

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraControlSS;

fn rc_camera_changed(
    q_camera: Query<(), (Changed<Transform>, With<GameCamera>)>,
) -> bool {
    !q_camera.is_empty()
}
