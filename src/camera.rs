use crate::prelude::*;
use crate::AppGlobalState;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(AppGlobalState::InGame, setup_camera);
        app.add_exit_system(AppGlobalState::InGame, despawn_with_recursive::<CameraCleanup>);
    }
}

#[derive(Component)]
struct CameraCleanup;

fn setup_camera(
    mut commands: Commands,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

