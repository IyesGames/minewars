use mw_common::grid::Pos;

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_event::<CameraJumpTo>();
    app.configure_stage_set(
        Update,
        CameraControlSS,
        rc_camera_changed,
    );
}

/// Marker for the main game camera (that renders the gameplay map view)
#[derive(Component)]
pub struct GameCamera;

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
