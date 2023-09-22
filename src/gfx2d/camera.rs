use crate::{prelude::*, camera::GameCamera};

use super::Gfx2dSet;

pub struct Gfx2dCameraPlugin;

impl Plugin for Gfx2dCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::InGame), setup_game_camera.in_set(Gfx2dSet::Any));
    }
}

fn setup_game_camera(
    world: &mut World,
) {
    let camera = Camera2dBundle::default();

    world.spawn((StateDespawnMarker, GameCamera, camera));
}
