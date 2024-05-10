use crate::prelude::*;

pub fn plugin(app: &mut App) {
}

/// Marker for the camera that displays our UI
#[derive(Component)]
pub struct UiCamera;

/// Marker for UI root entities / top-level containers
#[derive(Component)]
pub struct UiRoot;
