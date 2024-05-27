use mw_common::grid::Pos;

use crate::{input::*, prelude::*, ui::UiCamera};

pub fn plugin(app: &mut App) {
    app.add_event::<CameraJumpTo>();
    app.configure_stage_set(
        Update,
        CameraControlSS,
        rc_camera_changed,
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
#[derive(Component)]
pub struct ActiveGameCamera;

/// Event to cause a (smooth) jump to a given coordinate position
#[derive(Event)]
pub struct CameraJumpTo(pub Pos);

#[derive(SystemSet, Debug, PartialEq, Eq, Clone, Copy, Hash, Default)]
pub struct CameraControlSS;

#[derive(Bundle)]
pub struct CameraInputActionBundle {
    marker: CameraInput,
    bundle: InputActionBundle,
}

#[derive(Bundle)]
pub struct CameraInputAnalogBundle {
    marker: CameraInput,
    bundle: InputAnalogBundle,
}

#[derive(Component)]
pub struct CameraInput;

impl<'a> From<&'a str> for CameraInputActionBundle {
    fn from(value: &'a str) -> Self {
        Self {
            marker: CameraInput,
            bundle: value.into(),
        }
    }
}

impl<'a> From<&'a str> for CameraInputAnalogBundle {
    fn from(value: &'a str) -> Self {
        Self {
            marker: CameraInput,
            bundle: value.into(),
        }
    }
}

fn rc_camera_changed(
    q_camera: Query<(), (Changed<Transform>, With<GameCamera>)>,
) -> bool {
    !q_camera.is_empty()
}

pub mod input {
    use super::*;

    #[derive(Component)]
    pub struct AnalogPan;
    pub const ANALOG_PAN: &str = "ANALOG_CAMERA_PAN";
    #[derive(Component)]
    pub struct AnalogRotate;
    pub const ANALOG_ROTATE: &str = "ANALOG_CAMERA_ROTATE";
    #[derive(Component)]
    pub struct AnalogZoom;
    pub const ANALOG_ZOOM: &str = "ANALOG_CAMERA_ZOOM";

    pub(super) fn setup_inputs(mut commands: Commands) {
        commands.spawn((AnalogPan, CameraInputAnalogBundle::from(ANALOG_PAN)));
        commands.spawn((AnalogRotate, CameraInputAnalogBundle::from(ANALOG_ROTATE)));
        commands.spawn((AnalogZoom, CameraInputAnalogBundle::from(ANALOG_ZOOM)));
    }
}
