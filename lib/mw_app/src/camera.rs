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
        app.add_event::<ScreenShakeEvent>();
        app.init_resource::<GridCursor>();
        app.configure_sets(Update, (
            CameraControlSet
                .in_set(InGameSet(None)),
            GridCursorSet
                .in_set(InGameSet(None)),
            GridCursorChangedSet
                .after(GridCursorSet)
                .run_if(resource_changed::<GridCursor>())
        ));
    }
}

/// Marker for the main game camera (that renders the gameplay map view)
#[derive(Component)]
pub struct GameCamera;

/// Event to cause a (smooth) jump to a given coordinate position
#[derive(Event)]
pub struct CameraJumpTo(pub Pos);

/// Event to cause a screen shake
#[derive(Event)]
pub enum ScreenShakeEvent {
    Light,
    Medium,
    Strong,
}

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraControlSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GridCursorSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct GridCursorChangedSet;

#[derive(Resource, Default)]
pub struct GridCursor(pub Pos);
