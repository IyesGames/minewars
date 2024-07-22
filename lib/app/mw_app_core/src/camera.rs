use mw_common::grid::Pos;
use mw_ui_common::camera::UiCamera;

use crate::{input::*, prelude::*};

pub fn plugin(app: &mut App) {
    app.add_event::<CameraJumpTo>();
    app.configure_stage_set(
        Update,
        CameraControlSS,
        rc_camera_changed,
    );
    app.configure_stage_set(
        Update,
        CameraJumpSS,
        on_event::<CameraJumpTo>(),
    );
    app.add_systems(Startup, (
        input::setup_inputs
            .in_set(SetStage::Provide(GameInputSS::Setup)),
    ));
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
#[derive(Component, Default)]
pub struct ActiveGameCamera;

/// Event to cause a (smooth) jump to a given coordinate position
#[derive(Event)]
pub struct CameraJumpTo(pub Pos);

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraJumpSS;

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraControlSS;

fn rc_camera_changed(
    q_camera: Query<(), (Changed<Transform>, With<GameCamera>)>,
) -> bool {
    !q_camera.is_empty()
}

pub mod input {
    use super::*;

    #[derive(Component)]
    pub struct CameraInput;
    #[derive(Component)]
    pub struct CameraControlInput;

    #[derive(Component)]
    pub struct AnalogGridCursor;
    pub const ANALOG_GRID_CURSOR: &str = "ANALOG_GRID_CURSOR";
    #[derive(Component)]
    pub struct AnalogPan;
    pub const ANALOG_PAN: &str = "ANALOG_CAMERA_PAN";
    #[derive(Component)]
    pub struct AnalogRotate;
    pub const ANALOG_ROTATE: &str = "ANALOG_CAMERA_ROTATE";
    #[derive(Component)]
    pub struct AnalogZoom;
    pub const ANALOG_ZOOM: &str = "ANALOG_CAMERA_ZOOM";
    #[derive(Component)]
    pub struct ActionCenter;
    pub const ACTION_CENTER: &str = "ACTION_CAMERA_CENTER";

    pub(super) fn setup_inputs(mut commands: Commands) {
        commands.spawn((
            CameraInput, AnalogGridCursor,
            InputAnalogBundle::from(ANALOG_GRID_CURSOR),
        ));
        commands.spawn((
            CameraInput, CameraControlInput, AnalogPan,
            InputAnalogBundle::from(ANALOG_PAN),
        ));
        commands.spawn((
            CameraInput, CameraControlInput, AnalogRotate,
            InputAnalogBundle::from(ANALOG_ROTATE),
        ));
        commands.spawn((
            CameraInput, CameraControlInput, AnalogZoom,
            InputAnalogBundle::from(ANALOG_ZOOM),
        ));
        commands.spawn((
            CameraInput, CameraControlInput, ActionCenter,
            InputActionBundle::from(ACTION_CENTER),
        ));
    }
}
