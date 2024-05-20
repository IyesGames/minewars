use mw_app_core::{camera::GameCameraBundle, graphics::{Gfx2dEnabled, GraphicsGovernor}};

use crate::prelude::*;

pub fn plugin(app: &mut App) {
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_game_camera
            .run_if(any_filter::<(With<GraphicsGovernor>, With<Gfx2dEnabled>)>)
    );
}

fn setup_game_camera(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2dBundle::default(),
        GameCameraBundle::default(),
    ));
}
