//! General camera stuff
//!
//! Because we want to make the ALPHA playtesting prototype with 2D graphics,
//! and then transition to 3D graphics in future versions of the game,
//! any behavior related to the 2D game camera is in `gfx2d::camera`. This
//! file contains stuff that is agnostic between 2D/3D.

use mw_common::grid::Pos;

use crate::prelude::*;

pub struct MwCameraPlugin;

impl Plugin for MwCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CameraJumpTo>();
        app.init_resource::<GridCursor>();
        app.configure_stage_set(
            Update,
            CameraControlSS,
            rc_camera_changed,
        );
        app.configure_stage_set(
            Update,
            GridCursorSS,
            resource_changed::<GridCursor>,
        );
    }
}

/// Marker for the main game camera (that renders the gameplay map view)
#[derive(Component)]
pub struct GameCamera;

/// Event to cause a (smooth) jump to a given coordinate position
#[derive(Event)]
pub struct CameraJumpTo(pub Pos);

#[derive(Resource, Default)]
pub struct GridCursor(pub Pos);

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraControlSS;

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct GridCursorSS;

fn rc_camera_changed(
    q_camera: Query<(), (Changed<Transform>, With<GameCamera>)>,
) -> bool {
    !q_camera.is_empty()
}
